<!-- Full-screen barcode scanner modal. Uses the native BarcodeDetector API where
     available and falls back to a self-hosted zxing-wasm polyfill so scanning works
     offline (PWA) and across all major browsers. Emits `detected` with the raw value
     of the first barcode found, or `close` when the user dismisses the modal. -->
<template>
  <div class="modal modal-open">
    <div class="modal-box p-0 overflow-hidden">
      <div class="relative">
        <video ref="videoRef" autoplay playsinline muted class="w-full min-h-80 object-cover"></video>
        <div class="scanner-guide absolute inset-0 pointer-events-none flex items-center justify-center">
          <div class="guide-frame"></div>
        </div>
        <button
            class="btn btn-circle btn-sm absolute top-3 right-3 z-10 bg-black/50 border-none text-white"
            @click="$emit('close')"
        >
          <XMarkIcon class="size-5"/>
        </button>
        <div class="absolute bottom-0 inset-x-0 p-4 bg-linear-to-t from-black/70 to-transparent text-center">
          <p class="text-white text-sm">{{ statusMessage }}</p>
        </div>
      </div>
    </div>
    <div class="modal-backdrop" @click="$emit('close')"></div>
  </div>
</template>

<script lang="ts">
import {defineComponent, ref, onMounted, onBeforeUnmount} from 'vue';
import {XMarkIcon} from '@heroicons/vue/24/outline';
import {BarcodeDetector as BarcodeDetectorPolyfill, prepareZXingModule} from 'barcode-detector/pure';
import zxingWasmUrl from 'zxing-reader-wasm?url';

// Self-host the zxing-wasm binary (Vite emits it as a hashed asset) instead of the
// library's default jsdelivr CDN fetch — keeps scanning working offline (PWA) and avoids
// a runtime third-party dependency.
prepareZXingModule({
  overrides: {locateFile: () => zxingWasmUrl},
  equalityFn: Object.is,
  fireImmediately: false,
});

// Native BarcodeDetector where present (Chrome on macOS/Android/ChromeOS — Android uses
// ML Kit, the best decoder available). Everyone else (Firefox, Safari, desktop Chrome)
// gets the zxing-wasm ponyfill: far more reliable than the old pure-JS ZXing fallback,
// and the camera image is identical across browsers so behaviour is now consistent.
const Detector: any =
    'BarcodeDetector' in window ? (window as any).BarcodeDetector : BarcodeDetectorPolyfill;

export default defineComponent({
  components: {XMarkIcon},
  emits: ['detected', 'close'],
  setup(_, {emit}) {
    const statusMessage = ref('Point camera at a barcode…');
    const videoRef = ref<HTMLVideoElement | null>(null);
    let stream: MediaStream | null = null;
    let animationId = 0;

    const cameraConfigs: MediaStreamConstraints[] = [
      {video: {width: {min: 1280}, height: {min: 720}, facingMode: {ideal: 'environment'}}, audio: false},
      {video: {width: {ideal: 1920}, height: {ideal: 1080}, facingMode: {ideal: 'environment'}}, audio: false},
      {video: {facingMode: {ideal: 'environment'}}, audio: false},
      {video: true, audio: false},
    ];

    const startScanner = async () => {
      for (const config of cameraConfigs) {
        try {
          stream = await navigator.mediaDevices.getUserMedia(config);
          break;
        } catch {
        }
      }

      if (!stream) {
        statusMessage.value = 'Camera access denied or unavailable.';
        return;
      }

      const video = videoRef.value;
      if (!video) return;
      video.srcObject = stream;
      await video.play();

      const detector = new Detector({formats: ['ean_13', 'ean_8', 'upc_a', 'upc_e']});
      const detect = async () => {
        if (video.readyState < 2) {
          animationId = requestAnimationFrame(detect);
          return;
        }
        try {
          const codes = await detector.detect(video);
          if (codes.length > 0) {
            emit('detected', codes[0].rawValue);
            return;
          }
        } catch {
          // wasm still loading or no barcode this frame — keep scanning
        }
        animationId = requestAnimationFrame(detect);
      };
      animationId = requestAnimationFrame(detect);
    };

    const stopScanner = () => {
      if (animationId) cancelAnimationFrame(animationId);
      if (stream) {
        stream.getTracks().forEach((t) => t.stop());
        stream = null;
      }
    };

    onMounted(startScanner);
    onBeforeUnmount(stopScanner);

    return {statusMessage, videoRef};
  },
});
</script>

<style scoped>
.guide-frame {
  width: 80%;
  height: 30%;
  border: 2px solid rgba(255, 255, 255, 0.6);
  border-radius: 8px;
  box-shadow: 0 0 0 9999px rgba(0, 0, 0, 0.3);
}
</style>

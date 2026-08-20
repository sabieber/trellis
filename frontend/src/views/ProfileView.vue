<template>
  <PageContainer :title="$t('nav.profile')" ref="pageContainer">
    <template #title-button>
      <button
          @click="logout"
          class="flex items-center gap-1.5 text-sm font-semibold text-[#c98b6e] px-3 py-2 rounded-sm hover:bg-surface-2 transition-colors duration-150"
      >
        <PowerIcon class="size-5"/>
        {{ $t('common.logout') }}
      </button>
    </template>
    <div class="mt-4">
      <h3 class="t-eyebrow mb-2">{{ $t('profile.library') }}</h3>
      <div class="bg-surface border border-line rounded-md p-4">
        <p class="t-title text-[15px]">{{ $t('profile.importTitle') }}</p>
        <p class="t-meta mt-1 mb-3">{{ $t('profile.importDesc') }}</p>
        <div class="flex items-center gap-2">
          <input type="file" accept=".csv" @change="handleFileChange" class="file-input w-full max-w-xs"/>
          <Button :disabled="isUploading" @click="uploadFile">
            <span v-if="isUploading" class="loading loading-spinner loading-sm"></span>
            <span v-else>{{ $t('profile.upload') }}</span>
          </Button>
        </div>
        <label class="flex items-start gap-3 mt-3 cursor-pointer">
          <input type="checkbox" v-model="deriveReadingDays" class="checkbox checkbox-sm mt-0.5"/>
          <span>
            <span class="text-sm font-medium text-ink">{{ $t('profile.deriveReadingDays') }}</span>
            <span class="t-meta block">{{ $t('profile.deriveReadingDaysDesc') }}</span>
          </span>
        </label>
        <div v-if="importResult"
             :class="['mt-4 p-3 rounded-md border', importResult.success ? 'bg-success/10 border-success/30' : 'bg-error/10 border-error/30']">
          <div class="flex items-center gap-2">
            <span :class="importResult.success ? 'text-success' : 'text-error'">{{ importResult.message }}</span>
          </div>
        </div>
      </div>
    </div>

    <div class="mt-6">
      <h3 class="t-eyebrow mb-2">{{ $t('common.language') }}</h3>
      <div class="bg-surface border border-line rounded-md p-4">
        <SegmentedControl v-model="language" :options="languageOptions"/>
      </div>
    </div>

    <div class="mt-6">
      <h3 class="t-eyebrow mb-2">{{ $t('profile.rating') }}</h3>
      <div class="bg-surface border border-line rounded-md p-4">
        <p class="t-meta mb-3">{{ $t('profile.ratingModeDesc') }}</p>
        <SegmentedControl v-model="ratingModeSetting" :options="ratingModeOptions">
          <template #option="{ option }">
            <span class="flex items-center gap-1.5">
              <FlowerIcon v-if="option.value === 'stars'" class="size-4" fill="color-mix(in srgb, currentColor 50%, transparent)"/>
              <ThumbsUpIcon v-else class="size-4"/>
              {{ option.label }}
            </span>
          </template>
        </SegmentedControl>
      </div>
    </div>

    <div class="mt-6">
      <h3 class="t-eyebrow mb-2">{{ $t('profile.editionLanguages') }}</h3>
      <div class="bg-surface border border-line rounded-md p-4">
        <p class="t-meta mb-3">{{ $t('profile.editionLanguagesDesc') }}</p>
        <!-- Same control as the genres and tags on a book: chips to remove, type-ahead to add. -->
        <BookLabels
            :labels="selectedLanguageNames"
            :suggestions="languageSuggestions"
            :empty-text="$t('profile.allLanguages')"
            :add-label="$t('profile.addLanguage')"
            @add="addEditionLanguage"
            @remove="removeEditionLanguage"
        />
      </div>
    </div>

    <div class="mt-6">
      <h3 class="t-eyebrow mb-2">{{ $t('profile.app') }}</h3>
      <div class="bg-surface border border-line rounded-md p-4">
        <p class="t-title text-[15px]">{{ $t('profile.hardReloadTitle') }}</p>
        <p class="t-meta mt-1 mb-3">{{ $t('profile.hardReloadDesc') }}</p>
        <Button variant="ghost" :disabled="isReloading" @click="reloadWithoutCaches">
          <span v-if="isReloading" class="loading loading-spinner loading-sm"></span>
          <span v-else>{{ $t('profile.hardReloadAction') }}</span>
        </Button>
      </div>
    </div>
  </PageContainer>
</template>

<script lang="ts">
import {computed, defineComponent, ref} from 'vue';
import {useRouter} from 'vue-router';
import {useI18n} from 'vue-i18n';
import PageContainer from '@/components/PageContainer.vue';
import Button from '@/components/ui/Button.vue';
import SegmentedControl from '@/components/ui/SegmentedControl.vue';
import BookLabels from '@/components/BookLabels.vue';
import {FlowerIcon, PowerIcon, ThumbsUpIcon} from "@lucide/vue";
import {apiFetch} from '@/api/client';
import {useAuthStore} from '@/stores/auth';
import {setLocale, type Locale} from '@/i18n';
import {editionLanguageOptions, editionLanguages, languageLabel, setEditionLanguages} from '@/utils/editionLanguages';
import {ratingMode, setRatingMode, type RatingMode} from '@/utils/ratingMode';

export default defineComponent({
  components: {PageContainer, FlowerIcon, PowerIcon, ThumbsUpIcon, Button, SegmentedControl, BookLabels},
  setup() {
    const router = useRouter();
    const {t, locale} = useI18n();

    // Language names are shown as endonyms, so the labels don't get translated.
    const languageOptions = [{value: 'en', label: 'English'}, {value: 'de', label: 'Deutsch'}];
    const language = computed({
      get: () => locale.value,
      set: (value: string) => setLocale(value as Locale),
    });
    // No selection means no filter: a new reader sees every edition. The order
    // is the order they picked, and the edition lists follow it.
    //
    // BookLabels speaks in the names the user reads, so both directions go
    // through the option list of the current UI language. Names are unique
    // within a locale — the codes that share one are filtered out there.
    const options = computed(() => editionLanguageOptions(locale.value));
    const selectedLanguageNames = computed(() =>
        editionLanguages.value.map((code) => languageLabel(code, locale.value)));
    const languageSuggestions = computed(() => options.value.map((option) => option.label));

    // A typed name that matches no language is dropped: unlike a tag, this is a
    // pick from a fixed list, not free text.
    const addEditionLanguage = (name: string) => {
      const picked = options.value.find((option) => option.label === name);
      if (picked && !editionLanguages.value.includes(picked.code)) {
        setEditionLanguages([...editionLanguages.value, picked.code]);
      }
    };

    const removeEditionLanguage = (name: string) => setEditionLanguages(
        editionLanguages.value.filter((code) => languageLabel(code, locale.value) !== name));
    // Switching converts nothing: the stored 1..5 score stays, and each mode
    // renders it its own way.
    const ratingModeOptions = computed(() => [
      {value: 'stars', label: t('profile.ratingFlowers')},
      {value: 'thumbs', label: t('profile.ratingThumbs')},
    ]);
    const ratingModeSetting = computed({
      get: () => ratingMode.value,
      set: (value: string) => { void setRatingMode(value as RatingMode); },
    });

    const pageContainer = ref<any>(null);
    const selectedFile = ref<File | null>(null);
    const isUploading = ref(false);
    const importResult = ref<{ success: boolean; message: string } | null>(null);
    const deriveReadingDays = ref(false);
    const isReloading = ref(false);
    const auth = useAuthStore();

    // Escape hatch for a PWA that is stuck on an old build: drop the service
    // worker and everything it cached, then reload from the network.
    // ponytail: this cannot reach the browser's own HTTP cache. Serving
    // index.html with `Cache-Control: no-cache` is what fixes that half.
    const reloadWithoutCaches = async () => {
      isReloading.value = true;
      try {
        if ('serviceWorker' in navigator) {
          const registrations = await navigator.serviceWorker.getRegistrations();
          await Promise.all(registrations.map((registration) => registration.unregister()));
        }
        if ('caches' in window) {
          const keys = await caches.keys();
          await Promise.all(keys.map((key) => caches.delete(key)));
        }
      } finally {
        window.location.reload();
      }
    };

    const logout = () => {
      auth.logout();
      router.push('/login');
    };

    const handleFileChange = (event: Event) => {
      const input = event.target as HTMLInputElement;
      if (input.files && input.files[0]) {
        selectedFile.value = input.files[0];
      }
    };

    const uploadFile = async () => {
      if (!selectedFile.value) {
        pageContainer.value.showToast({message: t('profile.selectFileFirst'), type: 'alert-warning'});
        return;
      }

      isUploading.value = true;
      importResult.value = null;
      const formData = new FormData();
      formData.append('file', selectedFile.value);
      if (deriveReadingDays.value) {
        formData.append('derive_reading_days', 'true');
      }

      try {
        const response = await apiFetch('/api/user/import-good-reads', {
          method: 'POST',
          body: formData,
        });

        if (response.ok) {
          const d = await response.json();
          const summary = t('profile.importSummary', {
            added: d.books_added,
            skipped: d.books_skipped,
            readings: d.readings_created,
          });
          const failed = d.books_failed > 0 ? ' ' + t('profile.importFailed', {failed: d.books_failed}) : '';
          const days = d.entries_created > 0 ? ' ' + t('profile.importReadingDays', {days: d.entries_created}) : '';
          importResult.value = {success: true, message: summary + days + failed};
        } else {
          console.error('Failed to import file:', await response.json());
          importResult.value = {success: false, message: t('profile.importError')};
        }
      } catch (error) {
        console.error('Failed to import file:', error);
        importResult.value = {success: false, message: t('profile.importError')};
      } finally {
        isUploading.value = false;
      }
    };

    return {
      logout,
      pageContainer,
      handleFileChange,
      uploadFile,
      isUploading,
      importResult,
      deriveReadingDays,
      language,
      languageOptions,
      ratingModeSetting,
      ratingModeOptions,
      selectedLanguageNames,
      languageSuggestions,
      addEditionLanguage,
      removeEditionLanguage,
      isReloading,
      reloadWithoutCaches
    };
  }
});
</script>

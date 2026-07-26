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
        <div v-if="importResult" :class="['mt-4 p-3 rounded-md border', importResult.success ? 'bg-success/10 border-success/30' : 'bg-error/10 border-error/30']">
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
  </PageContainer>
</template>

<script lang="ts">
import {computed, defineComponent, ref} from 'vue';
import {useRouter} from 'vue-router';
import {useI18n} from 'vue-i18n';
import PageContainer from '@/components/PageContainer.vue';
import Button from '@/components/ui/Button.vue';
import SegmentedControl from '@/components/ui/SegmentedControl.vue';
import {PowerIcon} from "@heroicons/vue/24/outline";
import {apiFetch} from '@/api/client';
import {useAuthStore} from '@/stores/auth';
import {setLocale, type Locale} from '@/i18n';

export default defineComponent({
  components: {PageContainer, PowerIcon, Button, SegmentedControl},
  setup() {
    const router = useRouter();
    const {t, locale} = useI18n();

    // Language names are shown as endonyms, so the labels don't get translated.
    const languageOptions = [{value: 'en', label: 'English'}, {value: 'de', label: 'Deutsch'}];
    const language = computed({
      get: () => locale.value,
      set: (value: string) => setLocale(value as Locale),
    });
    const pageContainer = ref<any>(null);
    const selectedFile = ref<File | null>(null);
    const isUploading = ref(false);
    const importResult = ref<{ success: boolean; message: string } | null>(null);
    const deriveReadingDays = ref(false);
    const auth = useAuthStore();

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
          importResult.value = { success: true, message: summary + days + failed };
        } else {
          console.error('Failed to import file:', await response.json());
          importResult.value = { success: false, message: t('profile.importError') };
        }
      } catch (error) {
        console.error('Failed to import file:', error);
        importResult.value = { success: false, message: t('profile.importError') };
      } finally {
        isUploading.value = false;
      }
    };

    return {logout, pageContainer, handleFileChange, uploadFile, isUploading, importResult, deriveReadingDays, language, languageOptions};
  }
});
</script>

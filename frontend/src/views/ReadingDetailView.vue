<template>
  <PageContainer :title="$t('readingDetail.title')" ref="pageContainer">
    <div v-if="loading" class="flex justify-center">
      <span class="loading loading-spinner loading-lg"></span>
    </div>
    <div v-else>
      <div class="bg-surface border border-line rounded-md p-4 mb-5">
        <div class="flex justify-between items-center">
          <span class="t-meta">{{ $t('readingDetail.started') }}</span>
          <div v-if="editingStartDate" class="flex items-center gap-2">
            <input type="date" v-model="startDateDraft" class="input input-sm" />
            <button @click="saveStartDate" class="text-sm text-[#7a9e7e] hover:underline">{{ $t('common.save') }}</button>
            <button @click="editingStartDate = false" class="text-sm text-[#c47556] hover:underline">{{ $t('common.cancel') }}</button>
          </div>
          <div v-else class="flex items-center gap-2">
            <span class="text-sm text-ink">{{ startedAt }}</span>
            <button @click="beginEditStartDate" class="text-sm text-[#7a9e7e] hover:underline">{{ $t('common.edit') }}</button>
          </div>
        </div>
      </div>
      <div v-if="entries.length">
        <h2 class="t-eyebrow mb-3">{{ $t('readingDetail.activity') }}</h2>
        <ul class="flex flex-col gap-2.5">
          <li v-for="entry in entries" :key="entry.id" class="bg-surface border border-line rounded-md p-4">
            <div class="flex justify-between items-center">
              <span class="text-sm text-ink">{{ entry.read_at }}</span>
              <span class="t-meta">{{ mode === 'percentage' ? `${entry.progress}%` : $t('readingDetail.page', { n: entry.progress }) }}</span>
            </div>
            <p class="t-meta mt-1">{{ entry.mode }}</p>
          </li>
        </ul>
      </div>
      <div v-else class="t-meta text-center py-8">{{ $t('readingDetail.noEntries') }}</div>
      <Button block class="mt-5" @click="showModal = true">{{ $t('progressModal.title') }}</Button>
      <button
          @click="showAbandonConfirm = true"
          class="w-full mt-2 py-2.5 text-sm text-[#c47556] cursor-pointer hover:underline transition-colors duration-150"
      >
        {{ $t('readingDetail.abandonReading') }}
      </button>
      <button
          @click="showDeleteConfirm = true"
          class="w-full mt-2 py-2.5 text-sm text-[#c47556] cursor-pointer hover:underline transition-colors duration-150"
      >
        {{ $t('readingDetail.deleteReading') }}
      </button>
    </div>
    <TrackProgressModal v-if="showModal" @close="showModal = false" @submit="trackProgress" @abandon="onAbandonFromModal"
                        :initialProgress="latestProgress" :totalPages="totalPages" :mode="mode"/>
    <ConfirmDialog
        v-if="showAbandonConfirm"
        :title="$t('readingDetail.abandonTitle')"
        :message="$t('readingDetail.abandonMessage')"
        :confirmLabel="$t('common.abandon')"
        @confirm="abandonReading"
        @cancel="showAbandonConfirm = false"
    />
    <ConfirmDialog
        v-if="showDeleteConfirm"
        :title="$t('bookDetail.deleteReadingTitle')"
        :message="$t('bookDetail.deleteReadingMessage')"
        @confirm="deleteReading"
        @cancel="showDeleteConfirm = false"
    />
  </PageContainer>
</template>

<script lang="ts">
import {defineComponent, ref, onMounted} from 'vue';
import {useRoute, useRouter} from 'vue-router';
import {useI18n} from 'vue-i18n';
import TrackProgressModal from '@/components/TrackProgressModal.vue';
import ConfirmDialog from '@/components/ConfirmDialog.vue';
import PageContainer from '@/components/PageContainer.vue';
import Button from '@/components/ui/Button.vue';
import {apiFetch} from '@/api/client';
import {apiErrorMessage} from '@/utils/apiError';

export default defineComponent({
  components: {TrackProgressModal, ConfirmDialog, PageContainer, Button},
  setup() {
    const {t} = useI18n();
    const route = useRoute();
    const router = useRouter();
    const bookId = ref('');
    const entries = ref<Array<{ id: string, read_at: string, progress: number, mode: string }>>([]);
    const loading = ref(true);
    const showModal = ref(false);
    const showDeleteConfirm = ref(false);
    const showAbandonConfirm = ref(false);
    const latestProgress = ref(0);
    const totalPages = ref(0);
    const mode = ref('pages');
    const pageContainer = ref<any>(null);
    const startedAt = ref('');
    const editingStartDate = ref(false);
    const startDateDraft = ref('');

    const fetchReadingEntries = async (readingId: string) => {
      try {
        const response = await apiFetch('/api/books/reading', {
          method: 'POST',
          body: JSON.stringify({reading_id: readingId}),
        });
        if (response.ok) {
          const data = await response.json();
          entries.value = data.entries;
          bookId.value = data.book_id;
          startedAt.value = data.started_at;
          totalPages.value = data.total_pages ?? 0;
          mode.value = data.mode ?? 'pages';
          if (entries.value.length > 0) {
            latestProgress.value = entries.value[entries.value.length - 1].progress;
          }
        } else {
          console.error('Failed to fetch reading entries:', await response.json());
        }
      } catch (error) {
        console.error('Failed to fetch reading entries:', error);
      } finally {
        loading.value = false;
      }
    };

    const trackProgress = async (progress: number, readAt: string) => {
      try {
        const response = await apiFetch('/api/books/track-progress', {
          method: 'POST',
          body: JSON.stringify({reading_id: route.params.id, progress, read_at: readAt}),
        });
        if (response.ok) {
          fetchReadingEntries(route.params.id as string);
          showModal.value = false;
        } else {
          pageContainer.value?.showToast({message: apiErrorMessage(response.status, t), type: 'alert-error'});
        }
      } catch (error) {
        pageContainer.value?.showToast({message: t('error.network'), type: 'alert-error'});
      }
    };

    const deleteReading = async () => {
      showDeleteConfirm.value = false;
      try {
        const response = await apiFetch('/api/readings/delete', {
          method: 'POST',
          body: JSON.stringify({reading_id: route.params.id}),
        });
        if (response.ok) {
          router.replace({name: 'book-detail', params: {id: bookId.value}});
        } else {
          pageContainer.value?.showToast({message: apiErrorMessage(response.status, t), type: 'alert-error'});
        }
      } catch {
        pageContainer.value?.showToast({message: t('error.network'), type: 'alert-error'});
      }
    };

    const onAbandonFromModal = () => {
      showModal.value = false;
      showAbandonConfirm.value = true;
    };

    const abandonReading = async () => {
      showAbandonConfirm.value = false;
      try {
        const response = await apiFetch('/api/readings/cancel', {
          method: 'POST',
          body: JSON.stringify({reading_id: route.params.id}),
        });
        if (response.ok) {
          router.replace({name: 'book-detail', params: {id: bookId.value}});
        } else {
          pageContainer.value?.showToast({message: apiErrorMessage(response.status, t), type: 'alert-error'});
        }
      } catch {
        pageContainer.value?.showToast({message: t('error.network'), type: 'alert-error'});
      }
    };

    const beginEditStartDate = () => {
      startDateDraft.value = startedAt.value;
      editingStartDate.value = true;
    };

    const saveStartDate = async () => {
      try {
        const response = await apiFetch('/api/readings/update-started-at', {
          method: 'POST',
          body: JSON.stringify({reading_id: route.params.id, started_at: startDateDraft.value}),
        });
        if (response.ok) {
          startedAt.value = startDateDraft.value;
          editingStartDate.value = false;
        } else {
          pageContainer.value?.showToast({message: apiErrorMessage(response.status, t), type: 'alert-error'});
        }
      } catch {
        pageContainer.value?.showToast({message: t('error.network'), type: 'alert-error'});
      }
    };

    onMounted(() => {
      const readingId = route.params.id as string;
      fetchReadingEntries(readingId);
    });

    return {
      entries,
      loading,
      showModal,
      showDeleteConfirm,
      showAbandonConfirm,
      trackProgress,
      deleteReading,
      onAbandonFromModal,
      abandonReading,
      latestProgress,
      totalPages,
      mode,
      pageContainer,
      startedAt,
      editingStartDate,
      startDateDraft,
      beginEditStartDate,
      saveStartDate,
    };
  },
});
</script>

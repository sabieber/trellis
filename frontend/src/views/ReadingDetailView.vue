<template>
  <PageContainer title="Reading journal" ref="pageContainer">
    <div v-if="loading" class="flex justify-center">
      <span class="loading loading-spinner loading-lg"></span>
    </div>
    <div v-else-if="entries.length">
      <h2 class="t-eyebrow mb-3">Activity</h2>
      <ul class="flex flex-col gap-2.5">
        <li v-for="entry in entries" :key="entry.id" class="bg-surface border border-line rounded-md p-4">
          <div class="flex justify-between items-center">
            <span class="text-sm text-ink">{{ entry.read_at }}</span>
            <span class="t-meta">page {{ entry.progress }}</span>
          </div>
          <p class="t-meta mt-1">{{ entry.mode }}</p>
        </li>
      </ul>
    </div>
    <div v-else class="t-meta text-center py-8">No entries found.</div>
    <Button block class="mt-5" @click="showModal = true">Track Progress</Button>
    <button
        @click="showDeleteConfirm = true"
        class="w-full mt-2 py-2.5 text-sm text-[#c47556] cursor-pointer hover:underline transition-colors duration-150"
    >
      Delete reading
    </button>
    <TrackProgressModal v-if="showModal" @close="showModal = false" @submit="trackProgress"
                        :initialProgress="latestProgress"/>
    <ConfirmDialog
        v-if="showDeleteConfirm"
        title="Delete Reading"
        message="Are you sure you want to delete this reading session and all its entries? This cannot be undone."
        confirmLabel="Delete"
        @confirm="deleteReading"
        @cancel="showDeleteConfirm = false"
    />
  </PageContainer>
</template>

<script lang="ts">
import {defineComponent, ref, onMounted} from 'vue';
import {useRoute, useRouter} from 'vue-router';
import TrackProgressModal from '@/components/TrackProgressModal.vue';
import ConfirmDialog from '@/components/ConfirmDialog.vue';
import PageContainer from '@/components/PageContainer.vue';
import Button from '@/components/ui/Button.vue';
import {apiFetch} from '@/api/client';

export default defineComponent({
  components: {TrackProgressModal, ConfirmDialog, PageContainer, Button},
  setup() {
    const route = useRoute();
    const router = useRouter();
    const bookId = ref('');
    const entries = ref<Array<{ id: string, read_at: string, progress: number, mode: string }>>([]);
    const loading = ref(true);
    const showModal = ref(false);
    const showDeleteConfirm = ref(false);
    const latestProgress = ref(0);
    const pageContainer = ref<any>(null);

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
          const errorData = await response.json();
          pageContainer.value?.showToast({message: errorData.error, type: 'alert-error'});
        }
      } catch (error) {
        pageContainer.value?.showToast({message: 'Failed to track progress.', type: 'alert-error'});
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
          const errorData = await response.json();
          pageContainer.value?.showToast({message: errorData.error || 'Failed to delete reading.', type: 'alert-error'});
        }
      } catch {
        pageContainer.value?.showToast({message: 'Failed to delete reading.', type: 'alert-error'});
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
      trackProgress,
      deleteReading,
      latestProgress,
      pageContainer,
    };
  },
});
</script>

<template>
  <PageContainer title="Profile" ref="pageContainer">
    <template #title-button>
      <button
          @click="logout"
          class="flex items-center gap-1.5 text-sm font-semibold text-[#c98b6e] px-3 py-2 rounded-sm hover:bg-surface-2 transition-colors duration-150"
      >
        <PowerIcon class="size-5"/>
        Log out
      </button>
    </template>
    <div class="mt-4">
      <h3 class="t-eyebrow mb-2">Library</h3>
      <div class="bg-surface border border-line rounded-md p-4">
        <p class="t-title text-[15px]">Import from Goodreads</p>
        <p class="t-meta mt-1 mb-3">Select a CSV file to import your GoodReads data.</p>
        <div class="flex items-center gap-2">
          <input type="file" accept=".csv" @change="handleFileChange" class="file-input w-full max-w-xs"/>
          <Button :disabled="isUploading" @click="uploadFile">
            <span v-if="isUploading" class="loading loading-spinner loading-sm"></span>
            <span v-else>Upload</span>
          </Button>
        </div>
        <div v-if="importResult" :class="['mt-4 p-3 rounded-md border', importResult.success ? 'bg-success/10 border-success/30' : 'bg-error/10 border-error/30']">
          <div class="flex items-center gap-2">
            <span :class="importResult.success ? 'text-success' : 'text-error'">{{ importResult.message }}</span>
          </div>
        </div>
      </div>
    </div>
  </PageContainer>
</template>

<script lang="ts">
import {defineComponent, ref} from 'vue';
import {useRouter} from 'vue-router';
import PageContainer from '@/components/PageContainer.vue';
import Button from '@/components/ui/Button.vue';
import {PowerIcon} from "@heroicons/vue/24/outline";
import {apiFetch} from '@/api/client';
import {useAuthStore} from '@/stores/auth';

export default defineComponent({
  components: {PageContainer, PowerIcon, Button},
  setup() {
    const router = useRouter();
    const pageContainer = ref<any>(null);
    const selectedFile = ref<File | null>(null);
    const isUploading = ref(false);
    const importResult = ref<{ success: boolean; message: string } | null>(null);
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
        pageContainer.value.showToast({message: 'Please select a file first.', type: 'alert-warning'});
        return;
      }

      isUploading.value = true;
      importResult.value = null;
      const formData = new FormData();
      formData.append('file', selectedFile.value);

      try {
        const response = await apiFetch('/api/user/import-good-reads', {
          method: 'POST',
          body: formData,
        });

        if (response.ok) {
          const data = await response.json();
          importResult.value = { success: true, message: data.message };
        } else {
          console.error('Failed to import file:', await response.json());
          importResult.value = { success: false, message: 'Failed to import file.' };
        }
      } catch (error) {
        console.error('Failed to import file:', error);
        importResult.value = { success: false, message: 'Failed to import file.' };
      } finally {
        isUploading.value = false;
      }
    };

    return {logout, pageContainer, handleFileChange, uploadFile, isUploading, importResult};
  }
});
</script>

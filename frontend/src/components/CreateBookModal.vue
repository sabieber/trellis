<!-- Creates a book that no catalog knows (fan-fiction, a self-print, an obscure
     edition). It posts the same `/api/shelves/add-book` payload the search
     results use, only without any source id — the backend already treats those
     as optional. -->
<template>
  <div>
    <Button variant="soft" class="px-3.5! py-2! text-[13px]!" @click="open">
      <BookPlusIcon class="size-4"/>
      {{ $t('bookModal.newBook') }}
    </Button>
    <div v-if="show" class="modal modal-open">
      <form class="modal-box flex flex-col gap-4" @submit.prevent="createBook">
        <h3 class="t-title text-lg">{{ $t('bookModal.newBook') }}</h3>
        <div role="alert" class="alert alert-error" v-show="errorMessage">
          <TriangleAlertIcon class="size-5 shrink-0"/>
          <span v-text="errorMessage"></span>
        </div>
        <fieldset class="flex flex-col gap-1.5">
          <label class="t-meta">{{ $t('bookModal.bookTitle') }}</label>
          <input type="text" v-model="title" :placeholder="$t('bookModal.titlePlaceholder')" class="input w-full"
                 required/>
        </fieldset>
        <fieldset class="flex flex-col gap-1.5">
          <label class="t-meta">{{ $t('bookModal.author') }} <span class="text-faint">({{
              $t('common.optional')
            }})</span></label>
          <input type="text" v-model="author" :placeholder="$t('bookModal.authorPlaceholder')" class="input w-full"/>
        </fieldset>
        <fieldset class="flex flex-col gap-1.5">
          <label class="t-meta">{{ $t('common.pages') }} <span class="text-faint">({{
              $t('common.optional')
            }})</span></label>
          <input type="number" min="1" v-model="pageCount" class="input w-full"/>
        </fieldset>
        <fieldset class="flex flex-col gap-1.5">
          <label class="t-meta">{{ $t('bookModal.shelf') }}</label>
          <select v-model="shelfId" class="select w-full" required>
            <option v-for="shelf in shelves" :key="shelf.id" :value="shelf.id">{{ shelf.name || shelf.code }}</option>
          </select>
        </fieldset>
        <div class="modal-action mt-0 gap-2">
          <Button class="flex-1" :disabled="!title.trim() || !shelfId || submitting">{{ $t('common.create') }}</Button>
          <Button variant="ghost" type="button" @click="cancel">{{ $t('common.cancel') }}</Button>
        </div>
      </form>
      <div class="modal-backdrop" @click="cancel"></div>
    </div>
  </div>
</template>

<script lang="ts">
import {defineComponent, ref} from 'vue';
import {useI18n} from 'vue-i18n';
import {TriangleAlertIcon, BookPlusIcon} from '@lucide/vue';
import Button from '@/components/ui/Button.vue';
import {apiFetch} from '@/api/client';
import {apiErrorMessage} from '@/utils/apiError';

export default defineComponent({
  components: {TriangleAlertIcon, BookPlusIcon, Button},
  emits: ['bookCreated'],
  setup(_, {emit}) {
    const {t} = useI18n();
    const show = ref(false);
    const title = ref('');
    const author = ref('');
    const pageCount = ref<string>('');
    const shelfId = ref('');
    const shelves = ref<Array<{ id: string; code: string; name: string | null }>>([]);
    const submitting = ref(false);
    const errorMessage = ref('');

    const reset = () => {
      title.value = '';
      author.value = '';
      pageCount.value = '';
      errorMessage.value = '';
      submitting.value = false;
    };

    const open = async () => {
      reset();
      show.value = true;
      try {
        const response = await apiFetch('/api/shelves', {method: 'POST'});
        if (response.ok) {
          shelves.value = (await response.json()).shelves;
          shelfId.value = shelves.value[0]?.id ?? '';
        }
      } catch {
        errorMessage.value = t('error.network');
      }
    };

    const createBook = async () => {
      if (!title.value.trim() || !shelfId.value || submitting.value) return;
      submitting.value = true;
      try {
        const response = await apiFetch('/api/shelves/add-book', {
          method: 'POST',
          body: JSON.stringify({
            shelf_id: shelfId.value,
            title: title.value.trim(),
            author: author.value.trim() || null,
            page_count: Number(pageCount.value) || null,
          }),
        });
        if (response.ok) {
          show.value = false;
          reset();
          emit('bookCreated');
        } else {
          errorMessage.value = apiErrorMessage(response.status, t);
        }
      } catch {
        errorMessage.value = t('error.network');
      } finally {
        submitting.value = false;
      }
    };

    const cancel = () => {
      show.value = false;
      reset();
    };

    return {show, title, author, pageCount, shelfId, shelves, submitting, errorMessage, open, createBook, cancel};
  },
});
</script>

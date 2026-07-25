<template>
  <div>
    <Button variant="soft" class="px-3.5! py-2! text-[13px]!" @click="show = true">
      <PlusIcon class="size-4" />
      {{ $t('shelfModal.newShelf') }}
    </Button>
    <div v-if="show" class="modal modal-open">
      <div class="modal-box flex flex-col gap-4">
        <h3 class="t-title text-lg">{{ $t('shelfModal.newShelf') }}</h3>
        <div role="alert" class="alert alert-error" v-show="errorMessage">
          <ExclamationTriangleIcon class="size-5 shrink-0"/>
          <span v-text="errorMessage"></span>
        </div>
        <fieldset class="flex flex-col gap-1.5">
          <label class="t-meta">{{ $t('common.name') }}</label>
          <input type="text" v-model="name" :placeholder="$t('shelfModal.namePlaceholder')" class="input w-full" required/>
        </fieldset>
        <fieldset class="flex flex-col gap-1.5">
          <label class="t-meta">{{ $t('common.description') }} <span class="text-faint">({{ $t('common.optional') }})</span></label>
          <input type="text" v-model="description" :placeholder="$t('shelfModal.descPlaceholder')" class="input w-full"/>
        </fieldset>
        <div class="modal-action mt-0 gap-2">
          <Button class="flex-1" :disabled="!name" @click="createShelf">{{ $t('common.create') }}</Button>
          <Button variant="ghost" @click="cancel">{{ $t('common.cancel') }}</Button>
        </div>
      </div>
      <div class="modal-backdrop" @click="cancel"></div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { ExclamationTriangleIcon, PlusIcon } from "@heroicons/vue/24/outline";
import Button from '@/components/ui/Button.vue';
import { apiFetch } from '@/api/client';
import { apiErrorMessage } from '@/utils/apiError';

export default defineComponent({
  components: { ExclamationTriangleIcon, PlusIcon, Button },
  setup(_, { emit }) {
    const { t } = useI18n();
    const show = ref(false);
    const name = ref('');
    const description = ref('');
    const errorMessage = ref('');

    const createShelf = async () => {
      try {
        const response = await apiFetch('/api/shelves/create', {
          method: 'POST',
          body: JSON.stringify({ name: name.value, description: description.value }),
        });
        if (response.ok) {
          emit('shelfCreated');
          name.value = '';
          description.value = '';
          show.value = false;
          errorMessage.value = '';
        } else {
          errorMessage.value = apiErrorMessage(response.status, t);
        }
      } catch (error) {
        errorMessage.value = t('error.network');
      }
    };

    const cancel = () => {
      name.value = '';
      description.value = '';
      show.value = false;
      errorMessage.value = '';
    };

    return { show, name, description, errorMessage, createShelf, cancel };
  },
});
</script>

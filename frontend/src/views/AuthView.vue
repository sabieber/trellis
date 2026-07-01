<template>
  <div class="min-h-screen lattice-bg flex flex-col items-center justify-center px-6 py-12">
    <div class="flex flex-col items-center mb-8">
      <img src="/logo.svg" class="size-20 rounded-full shadow-soft" alt="trellis"/>
      <h1 class="t-display text-4xl mt-4">trellis</h1>
      <p class="t-italic text-green-soft text-lg mt-1">a garden library</p>
      <p class="t-meta text-center max-w-65 mt-3.5 leading-relaxed">
        {{
          activeTab === 'signup' ? 'Plant your shelves and watch a reading life take root.' : 'Welcome back — your books have been waiting.'
        }}
      </p>
    </div>

    <div v-if="errorMessage" role="alert" class="alert alert-error mb-6 w-full max-w-sm">
      <ExclamationTriangleIcon class="size-5 shrink-0"/>
      <span>{{ errorMessage }}</span>
    </div>

    <div class="w-full max-w-sm flex flex-col gap-4">
      <div class="flex flex-col gap-1">
        <label class="t-meta">Name</label>
        <input
            type="text"
            class="input w-full"
            placeholder="you@example.com"
            v-model="username"
            autocomplete="username"
        />
      </div>

      <div class="flex flex-col gap-1">
        <div class="flex justify-between items-center">
          <label class="t-meta">Password</label>
          <a v-if="activeTab === 'signin'" class="t-meta text-green-soft hover:text-green transition-colors duration-150 cursor-pointer">Forgot password?</a>
        </div>
        <input
            type="password"
            class="input w-full"
            placeholder="••••••••"
            v-model="password"
            autocomplete="current-password"
            @keydown.enter="submit"
        />
      </div>

      <Button block class="mt-2" @click="submit">
        {{ activeTab === 'signin' ? 'Log in' : 'Create account' }}
      </Button>

      <p class="t-meta text-center mt-4">
        {{ activeTab === 'signin' ? 'New to trellis?' : 'Already growing a library?' }}
        <button
            class="text-green-soft font-semibold hover:text-green transition-colors duration-150 cursor-pointer ml-1"
            @click="switchTab(activeTab === 'signin' ? 'signup' : 'signin')"
        >
          {{ activeTab === 'signin' ? 'Create an account' : 'Log in' }}
        </button>
      </p>
    </div>
  </div>
</template>

<script lang="ts">
import {defineComponent, ref, watch} from 'vue';
import {useRoute} from 'vue-router';
import {ExclamationTriangleIcon} from '@heroicons/vue/24/outline';
import router from '@/router';
import {useAuthStore} from '@/stores/auth';
import {API_BASE_URL} from '@/api/config';
import Button from '@/components/ui/Button.vue';

export default defineComponent({
  components: {ExclamationTriangleIcon, Button},
  setup() {
    const route = useRoute();
    const activeTab = ref<'signin' | 'signup'>(route.name === 'register' ? 'signup' : 'signin');
    const username = ref('');
    const password = ref('');
    const errorMessage = ref('');
    const auth = useAuthStore();

    watch(() => route.name, (name) => {
      activeTab.value = name === 'register' ? 'signup' : 'signin';
      errorMessage.value = '';
    });

    const switchTab = (tab: 'signin' | 'signup') => {
      router.push(tab === 'signin' ? '/login' : '/register');
    };

    const login = async () => {
      try {
        const response = await fetch(`${API_BASE_URL}/api/user/login`, {
          method: 'POST',
          headers: {'Content-Type': 'application/json'},
          body: JSON.stringify({username: username.value, password: password.value}),
        });
        if (response.ok) {
          const data: { token: string; user_id: string } = await response.json();
          auth.setAuth(data.token, data.user_id);
          router.push('/');
        } else {
          const data: { error: string } = await response.json();
          errorMessage.value = data.error;
        }
      } catch {
        errorMessage.value = 'Failed to connect to the server!';
      }
    };

    const register = async () => {
      try {
        const response = await fetch(`${API_BASE_URL}/api/user/register`, {
          method: 'POST',
          headers: {'Content-Type': 'application/json'},
          body: JSON.stringify({username: username.value, password: password.value}),
        });
        if (response.ok) {
          await login();
        } else {
          const data: { error: string } = await response.json();
          errorMessage.value = data.error;
        }
      } catch {
        errorMessage.value = 'Failed to connect to the server!';
      }
    };

    const submit = () => {
      errorMessage.value = '';
      if (activeTab.value === 'signin') {
        login();
      } else {
        register();
      }
    };

    return {activeTab, username, password, errorMessage, switchTab, submit};
  },
});
</script>

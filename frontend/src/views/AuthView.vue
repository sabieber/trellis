<template>
  <div class="min-h-screen bg-gray-900 flex flex-col items-center justify-center px-6 py-12">
    <div v-if="errorMessage" role="alert" class="alert alert-error mb-6 w-full max-w-sm">
      <ExclamationTriangleIcon class="size-5 shrink-0 text-white"/>
      <span>{{ errorMessage }}</span>
    </div>

    <div class="flex flex-col items-center mb-8">
      <img src="/logo.svg" class="size-16 mb-4" alt="trellis" />
      <h1 class="text-2xl font-bold text-white">trellis</h1>
      <p class="text-gray-400 text-sm mt-1">grow your library and reading goals</p>
    </div>

    <div role="tablist" class="tabs tabs-box w-full max-w-sm mb-6">
      <button role="tab" class="tab flex-1" :class="{ 'tab-active': activeTab === 'signin' }" @click="switchTab('signin')">
        Sign In
      </button>
      <button role="tab" class="tab flex-1" :class="{ 'tab-active': activeTab === 'signup' }" @click="switchTab('signup')">
        Sign Up
      </button>
    </div>

    <div class="w-full max-w-sm flex flex-col gap-4">
      <div class="flex flex-col gap-1">
        <label class="text-sm text-gray-400">Name</label>
        <input
          type="text"
          class="input input-bordered w-full"
          placeholder="you@example.com"
          v-model="username"
          autocomplete="username"
        />
      </div>

      <div class="flex flex-col gap-1">
        <div class="flex justify-between items-center">
          <label class="text-sm text-gray-400">Password</label>
          <a v-if="activeTab === 'signin'" class="text-sm text-primary cursor-pointer">Forgot password?</a>
        </div>
        <input
          type="password"
          class="input input-bordered w-full"
          placeholder="••••••••"
          v-model="password"
          autocomplete="current-password"
          @keydown.enter="submit"
        />
      </div>

      <button class="btn btn-primary btn-block mt-2" @click="submit">
        {{ activeTab === 'signin' ? 'Sign In' : 'Sign Up' }}
      </button>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, watch } from 'vue';
import { useRoute } from 'vue-router';
import { ExclamationTriangleIcon } from '@heroicons/vue/16/solid';
import router from '@/router';
import { useAuthStore } from '@/stores/auth';
import { API_BASE_URL } from '@/api/config';

export default defineComponent({
  components: { ExclamationTriangleIcon },
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
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ username: username.value, password: password.value }),
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
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ username: username.value, password: password.value }),
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

    return { activeTab, username, password, errorMessage, switchTab, submit };
  },
});
</script>

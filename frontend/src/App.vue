<template>
  <div class="flex flex-col min-h-screen">
    <div v-if="!$route.meta.hideNav" class="navbar bg-gray-800 fixed w-full z-10">
      <div class="flex-1">
        <a class="btn btn-ghost text-xl">
          <RouterLink to="/">Books</RouterLink>
        </a>
      </div>
      <div class="flex-none hidden sm:flex">
        <ul class="menu menu-horizontal px-1">
          <li class="tooltip tooltip-bottom" data-tip="Home">
            <RouterLink to="/" :class="{ 'menu-active': $route.path === '/' }">
              <HomeIcon class="size-6" />
            </RouterLink>
          </li>
          <li class="tooltip tooltip-bottom" data-tip="Search">
            <RouterLink to="/search" :class="{ 'menu-active': $route.path === '/search' }">
              <MagnifyingGlassIcon class="size-6" />
            </RouterLink>
          </li>
          <li class="tooltip tooltip-bottom" data-tip="Library">
            <RouterLink to="/library" :class="{ 'menu-active': $route.path === '/library' }">
              <BookOpenIcon class="size-6" />
            </RouterLink>
          </li>
          <li class="tooltip tooltip-bottom" data-tip="Profile">
            <RouterLink to="/profile" :class="{ 'menu-active': $route.path === '/profile' }">
              <UserIcon class="size-6" />
            </RouterLink>
          </li>
        </ul>
      </div>
    </div>
    
    <RouterView class="flex-grow" :class="{ 'pt-16 pb-16': !$route.meta.hideNav }" />

    <div v-if="!$route.meta.hideNav" class="dock dock-sm sm:hidden">
      <button :class="{ 'dock-active': $route.path === '/' }">
        <RouterLink to="/">
          <HomeIcon class="size-6" />
        </RouterLink>
      </button>
      <button :class="{ 'dock-active': $route.path === '/search' }">
        <RouterLink to="/search">
          <MagnifyingGlassIcon class="size-6" />
        </RouterLink>
      </button>
      <button :class="{ 'dock-active': $route.path === '/library' }">
        <RouterLink to="/library">
          <BookOpenIcon class="size-6" />
        </RouterLink>
      </button>
      <button :class="{ 'dock-active': $route.path === '/profile' }">
        <RouterLink to="/profile">
          <UserIcon class="size-6" />
        </RouterLink>
      </button>
    </div>
  </div>
</template>

<script lang="ts">
import { MagnifyingGlassIcon, HomeIcon, BookOpenIcon, UserIcon } from "@heroicons/vue/16/solid";

export default {
  components: { MagnifyingGlassIcon, HomeIcon, BookOpenIcon, UserIcon },
};
</script>

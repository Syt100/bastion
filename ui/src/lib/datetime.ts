import { computed, unref, type ComputedRef, type Ref } from 'vue'

type MaybeRef<T> = T | Ref<T> | ComputedRef<T>

export function useUnixSecondsFormatter(locale: MaybeRef<string>) {
  const formatter = computed(
    () =>
      new Intl.DateTimeFormat(unref(locale), {
        dateStyle: 'medium',
        timeStyle: 'medium',
      }),
  )

  function formatUnixSeconds(ts: number | null): string {
    if (!ts) return '-'
    return formatter.value.format(new Date(ts * 1000))
  }

  return { formatUnixSeconds }
}


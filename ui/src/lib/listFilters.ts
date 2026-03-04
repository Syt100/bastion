import { computed, type ComputedRef, type Ref } from 'vue'

import type { FilterChip } from '@/lib/filterChips'

type Primitive = string | number | boolean

type FilterOption<T extends Primitive> = {
  label: string
  value: T
}

type FilterOptionsSource<T extends Primitive> =
  | ComputedRef<FilterOption<T>[]>
  | Ref<FilterOption<T>[]>
  | (() => FilterOption<T>[])

type ListFilterField = {
  clear: () => void
  isActive: () => boolean
  chips: () => FilterChip[]
}

function resolveOptions<T extends Primitive>(source: FilterOptionsSource<T>): FilterOption<T>[] {
  if (typeof source === 'function') return source()
  return source.value
}

function findOptionLabel<T extends Primitive>(source: FilterOptionsSource<T>, value: T): string {
  return resolveOptions(source).find((it) => it.value === value)?.label ?? String(value)
}

function cloneArrayOrNull<T extends Primitive>(value: T[] | null | undefined): T[] | null {
  if (value == null) return null
  return value.slice()
}

function normalizedMultiValues<T extends Primitive>(value: T[] | null | undefined): T[] {
  if (!Array.isArray(value)) return []
  return value.filter((item, idx, arr) => arr.indexOf(item) === idx)
}

export function createTextFilterField(params: {
  key: string
  label: string
  value: Ref<string>
  defaultValue?: string
  normalize?: (value: string) => string
}): ListFilterField {
  const normalize = params.normalize ?? ((value: string) => value.trim())
  const defaultValue = params.defaultValue ?? ''

  return {
    clear: () => {
      params.value.value = defaultValue
    },
    isActive: () => normalize(params.value.value).length > 0,
    chips: () => {
      const value = normalize(params.value.value)
      if (value.length === 0) return []
      return [
        {
          key: params.key,
          label: `${params.label}: ${value}`,
          onClose: () => {
            params.value.value = defaultValue
          },
        },
      ]
    },
  }
}

export function createSingleSelectFilterField<T extends Primitive>(params: {
  key: string
  label: string
  value: Ref<T>
  defaultValue: T
  options: FilterOptionsSource<T>
  chipLabel?: (value: T, optionLabel: string) => string
}): ListFilterField {
  return {
    clear: () => {
      params.value.value = params.defaultValue
    },
    isActive: () => params.value.value !== params.defaultValue,
    chips: () => {
      const value = params.value.value
      if (value === params.defaultValue) return []
      const optionLabel = findOptionLabel(params.options, value)
      const label = params.chipLabel ? params.chipLabel(value, optionLabel) : `${params.label}: ${optionLabel}`

      return [
        {
          key: params.key,
          label,
          onClose: () => {
            params.value.value = params.defaultValue
          },
        },
      ]
    },
  }
}

export function createMultiSelectFilterField<T extends Primitive>(params: {
  key: string
  label: string
  value: Ref<T[] | null | undefined>
  options: FilterOptionsSource<T>
  defaultValue?: T[] | null
  chipLabel?: (value: T, optionLabel: string) => string
}): ListFilterField {
  const defaultValue = cloneArrayOrNull(params.defaultValue ?? [])

  function clearValue(): void {
    params.value.value = cloneArrayOrNull(defaultValue)
  }

  function setValues(values: T[]): void {
    if (values.length === 0) {
      clearValue()
      return
    }
    params.value.value = values
  }

  return {
    clear: clearValue,
    isActive: () => normalizedMultiValues(params.value.value).length > 0,
    chips: () => {
      const values = normalizedMultiValues(params.value.value)
      if (values.length === 0) return []

      return values.map((value, index) => {
        const optionLabel = findOptionLabel(params.options, value)
        const label = params.chipLabel
          ? params.chipLabel(value, optionLabel)
          : `${params.label}: ${optionLabel}`

        return {
          key: `${params.key}:${String(value)}:${index}`,
          label,
          onClose: () => {
            const next = normalizedMultiValues(params.value.value).filter((item) => item !== value)
            setValues(next)
          },
        }
      })
    },
  }
}

export function useListFilters(fields: ListFilterField[]) {
  const filtersActiveCount = computed<number>(() => {
    let count = 0
    for (const field of fields) {
      if (field.isActive()) count += 1
    }
    return count
  })

  const hasActiveFilters = computed<boolean>(() => filtersActiveCount.value > 0)

  const activeFilterChips = computed<FilterChip[]>(() => {
    const chips: FilterChip[] = []
    for (const field of fields) {
      chips.push(...field.chips())
    }
    return chips
  })

  function clearFilters(): void {
    for (const field of fields) {
      field.clear()
    }
  }

  return {
    filtersActiveCount,
    hasActiveFilters,
    activeFilterChips,
    clearFilters,
  }
}

export function parseRouteQueryList(value: unknown): string[] {
  const split = (raw: string): string[] =>
    raw
      .split(',')
      .map((item) => item.trim())
      .filter((item) => item.length > 0)

  if (Array.isArray(value)) {
    return value.filter((item): item is string => typeof item === 'string').flatMap(split)
  }
  if (typeof value === 'string') return split(value)
  return []
}

export function parseRouteQueryFirst(value: unknown): string | null {
  const values = parseRouteQueryList(value)
  return values[0] ?? null
}

export function parseRouteQueryEnum<T extends string>(
  value: unknown,
  allowedValues: readonly T[],
  fallback: T,
): T {
  const candidates = parseRouteQueryList(value)
  for (const candidate of candidates) {
    if (allowedValues.includes(candidate as T)) {
      return candidate as T
    }
  }
  return fallback
}

export function parseRouteQueryBoolean(
  value: unknown,
  fallback = false,
): boolean {
  const candidate = parseRouteQueryFirst(value)
  if (!candidate) return fallback
  const normalized = candidate.toLowerCase()
  if (normalized === '1' || normalized === 'true' || normalized === 'yes' || normalized === 'on') return true
  if (normalized === '0' || normalized === 'false' || normalized === 'no' || normalized === 'off') return false
  return fallback
}

import type { JobSimpleScheduleKind } from './types'

export type SimpleScheduleState = {
  kind: JobSimpleScheduleKind
  everyMinutes: number
  atHour: number
  atMinute: number
  weekday: number
  monthday: number
}

function clampInt(value: number, min: number, max: number): number {
  const v = Math.floor(Number.isFinite(value) ? value : min)
  return Math.min(max, Math.max(min, v))
}

export function simpleScheduleToCron(state: SimpleScheduleState): string {
  const everyMinutes = clampInt(state.everyMinutes, 1, 59)
  const atHour = clampInt(state.atHour, 0, 23)
  const atMinute = clampInt(state.atMinute, 0, 59)
  const weekday = clampInt(state.weekday, 0, 6)
  const monthday = clampInt(state.monthday, 1, 28)

  switch (state.kind) {
    case 'every_minutes':
      return `*/${everyMinutes} * * * *`
    case 'hourly':
      return `${atMinute} * * * *`
    case 'daily':
      return `${atMinute} ${atHour} * * *`
    case 'weekly':
      return `${atMinute} ${atHour} * * ${weekday}`
    case 'monthly':
      return `${atMinute} ${atHour} ${monthday} * *`
  }
}

export function cronToSimpleSchedule(expr: string): SimpleScheduleState | null {
  const parts = expr
    .trim()
    .split(/\s+/g)
    .map((v) => v.trim())
    .filter(Boolean)
  if (parts.length !== 5) return null

  const [min, hour, dom, month, dow] = parts as [string, string, string, string, string]

  // Every N minutes: */N * * * *
  if (hour === '*' && dom === '*' && month === '*' && dow === '*') {
    if (/^\*\/\d+$/.test(min)) {
      const everyMinutes = Number(min.slice(2))
      if (!Number.isFinite(everyMinutes) || everyMinutes < 1 || everyMinutes > 59) return null
      return {
        kind: 'every_minutes',
        everyMinutes,
        atHour: 0,
        atMinute: 0,
        weekday: 1,
        monthday: 1,
      }
    }

    if (/^\d+$/.test(min)) {
      const atMinute = Number(min)
      if (atMinute < 0 || atMinute > 59) return null
      return {
        kind: 'hourly',
        everyMinutes: 15,
        atHour: 0,
        atMinute,
        weekday: 1,
        monthday: 1,
      }
    }
  }

  // Daily: M H * * *
  if (dom === '*' && month === '*' && dow === '*' && /^\d+$/.test(min) && /^\d+$/.test(hour)) {
    const atMinute = Number(min)
    const atHour = Number(hour)
    if (atMinute < 0 || atMinute > 59 || atHour < 0 || atHour > 23) return null
    return {
      kind: 'daily',
      everyMinutes: 15,
      atHour,
      atMinute,
      weekday: 1,
      monthday: 1,
    }
  }

  // Weekly: M H * * D
  if (dom === '*' && month === '*' && /^\d+$/.test(min) && /^\d+$/.test(hour) && /^\d+$/.test(dow)) {
    const atMinute = Number(min)
    const atHour = Number(hour)
    const weekday = Number(dow === '7' ? '0' : dow)
    if (atMinute < 0 || atMinute > 59 || atHour < 0 || atHour > 23 || weekday < 0 || weekday > 6) return null
    return {
      kind: 'weekly',
      everyMinutes: 15,
      atHour,
      atMinute,
      weekday,
      monthday: 1,
    }
  }

  // Monthly: M H D * *
  if (month === '*' && dow === '*' && /^\d+$/.test(min) && /^\d+$/.test(hour) && /^\d+$/.test(dom)) {
    const atMinute = Number(min)
    const atHour = Number(hour)
    const monthday = Number(dom)
    if (atMinute < 0 || atMinute > 59 || atHour < 0 || atHour > 23) return null
    // Keep simple mode conservative to avoid "missing days" in short months.
    if (monthday < 1 || monthday > 28) return null
    return {
      kind: 'monthly',
      everyMinutes: 15,
      atHour,
      atMinute,
      weekday: 1,
      monthday,
    }
  }

  return null
}

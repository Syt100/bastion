export type UiBuildInfo = {
  version: string
  buildTimeUnix: number
  gitSha: string | null
}

export const UI_BUILD_INFO: UiBuildInfo = {
  version: __BASTION_UI_VERSION__,
  buildTimeUnix: __BASTION_UI_BUILD_TIME_UNIX__,
  gitSha: __BASTION_UI_GIT_SHA__,
}


export type SendKeyMode = 'enter' | 'ctrl-enter';
export type MotionSpeed = 'expressive' | 'snappy' | 'off';

export interface GeneralPreferences {
  sendKey: SendKeyMode;
  escToHide: boolean;
  showWelcomeMark: boolean;
  motionSpeed: MotionSpeed;
}

export const DEFAULT_GENERAL_PREFERENCES: GeneralPreferences = {
  sendKey: 'enter',
  escToHide: true,
  showWelcomeMark: true,
  motionSpeed: 'expressive',
};

export const GENERAL_PREFS_KEY = 'zwen.general-prefs.v1';

export function parseGeneralPreferences(value: unknown): GeneralPreferences {
  if (!value || typeof value !== 'object') return { ...DEFAULT_GENERAL_PREFERENCES };
  const v = value as Record<string, unknown>;
  return {
    sendKey: v.sendKey === 'ctrl-enter' ? 'ctrl-enter' : 'enter',
    escToHide: typeof v.escToHide === 'boolean' ? v.escToHide : true,
    showWelcomeMark: typeof v.showWelcomeMark === 'boolean' ? v.showWelcomeMark : true,
    motionSpeed: v.motionSpeed === 'snappy' || v.motionSpeed === 'off' ? v.motionSpeed : 'expressive',
  };
}

export function loadGeneralPreferences(storage: Pick<Storage, 'getItem'> = localStorage): GeneralPreferences {
  try {
    return parseGeneralPreferences(JSON.parse(storage.getItem(GENERAL_PREFS_KEY) ?? 'null'));
  } catch {
    return { ...DEFAULT_GENERAL_PREFERENCES };
  }
}

export function saveGeneralPreferences(prefs: GeneralPreferences, storage: Pick<Storage, 'setItem'> = localStorage) {
  try {
    storage.setItem(GENERAL_PREFS_KEY, JSON.stringify(prefs));
  } catch {
    /* ignore storage errors */
  }
}

export function applyGeneralPreferences(prefs: GeneralPreferences) {
  document.documentElement.dataset.motion = prefs.motionSpeed;
}

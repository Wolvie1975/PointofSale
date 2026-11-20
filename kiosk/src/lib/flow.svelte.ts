import { staffApi, type BookingConfirmation, type ClientDetails, type Minutes } from './api';

export type Screen =
  | 'home' | 'service' | 'time' | 'calendar' | 'details' | 'done'
  | 'staff' | 'staffClients' | 'staffClient';
export type TextSize = 'small' | 'medium' | 'large';
/** On-screen keyboard layout, or null when hidden. */
export type KeyboardMode = 'text' | 'phone' | null;

export const isStaffScreen = (s: Screen) => s.startsWith('staff');

function initial() {
  return {
    screen: 'home' as Screen,
    textSize: 'medium' as TextSize,
    textSizeOpen: false, // TextSizePicker visible
    staffSignInOpen: false, // staff PIN pad visible
    keyboard: null as KeyboardMode,
    // Client booking
    serviceId: null as number | null,
    minutes: 60 as Minutes,
    date: '', // 'YYYY-MM-DD'
    start: '', // 'HH:MM'
    startLabel: '', // '4:00 PM'
    client: { name: '', phone: '', email: '' } as ClientDetails,
    confirmation: null as BookingConfirmation | null, // set when the booking is created
    // Staff
    staffToken: null as string | null, // from staffApi.signIn; every staff call needs it
    staffClientId: null as string | null,
  };
}

/** Everything the current screen user has chosen. One person at a time. */
export const flow = $state(initial());

/** Clears everything, including text size and any staff session, for the next person. */
export function resetFlow() {
  const token = flow.staffToken;
  Object.assign(flow, initial());
  if (token) staffApi.signOut(token).catch(() => {}); // the backend also ends it after 5 min idle
}

/** Where Back (and Esc) goes from each screen. */
export const backFrom: Partial<Record<Screen, Screen>> = {
  service: 'home',
  time: 'service',
  calendar: 'time',
  details: 'time',
  staffClients: 'staff',
  staffClient: 'staffClients',
};

$effect.root(() => {
  $effect(() => {
    document.documentElement.dataset.textSize = flow.textSize;
  });
});

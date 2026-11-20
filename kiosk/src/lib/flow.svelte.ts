import type { BookingConfirmation, ClientDetails, Minutes } from './api';

export type Screen = 'home' | 'service' | 'time' | 'calendar' | 'details' | 'done';
export type TextSize = 'small' | 'medium' | 'large';
export type Field = keyof ClientDetails;

function initial() {
  return {
    screen: 'home' as Screen,
    textSize: 'medium' as TextSize,
    textSizeOpen: false, // TextSizePicker visible
    staffOpen: false, // staff PIN pad (close app) visible
    serviceId: null as number | null,
    minutes: 60 as Minutes,
    date: '', // 'YYYY-MM-DD'
    start: '', // 'HH:MM'
    startLabel: '', // '4:00 PM'
    client: { name: '', phone: '', email: '' } as ClientDetails,
    activeField: null as Field | null,
    confirmation: null as BookingConfirmation | null, // set when the booking is created
  };
}

/** Everything the current client has chosen. One client at a time. */
export const flow = $state(initial());

/** Clears everything, including text size, for the next client. */
export function resetFlow() {
  Object.assign(flow, initial());
}

/** Where Back (and Esc) goes from each screen. */
export const backFrom: Partial<Record<Screen, Screen>> = {
  service: 'home',
  time: 'service',
  calendar: 'time',
  details: 'time',
};

$effect.root(() => {
  $effect(() => {
    document.documentElement.dataset.textSize = flow.textSize;
  });
});

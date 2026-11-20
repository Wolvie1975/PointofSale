import { invoke } from '@tauri-apps/api/core';

// Mirrors kiosk-core/src/domain.rs (serialized in camelCase).
export type SlotStatus = 'open' | 'unavailable';
export type DayStatus = 'past' | 'closed' | 'open' | 'limited' | 'full';
export type Minutes = 30 | 60 | 90;

export interface ServiceOption { minutes: Minutes; priceCents: number }
export interface Service { id: number; name: string; description: string; options: ServiceOption[] }
export interface Slot { start: string; label: string; status: SlotStatus }
export interface DayAvailability { date: string; status: DayStatus }
export interface ClientDetails { name: string; phone: string; email: string }
export interface BookingConfirmation {
  id: string; serviceName: string; minutes: Minutes; date: string; startLabel: string; priceCents: number;
}

// Staff-only types. Times are local 'YYYY-MM-DDTHH:MM:SS' (chrono NaiveDateTime).
export type PaymentStatus = 'unpaid' | 'paid';
export interface Client { id: string; name: string; phone: string; email: string }
export interface Note { id: string; clientId: string; text: string; createdAt: string; editedAt: string | null }
export interface BookingView {
  id: string; serviceName: string; minutes: Minutes; start: string; priceCents: number;
  payment: PaymentStatus; paidAt: string | null;
}
export interface ClientSummary {
  id: string; name: string; phone: string; email: string;
  bookings: number; unpaid: number; notes: number; nextVisit: string | null;
}
export interface ClientRecord { client: Client; bookings: BookingView[]; notes: Note[] }

// Tauri maps camelCase argument keys to the Rust snake_case parameter names.
// Failed calls reject with the client-facing message from kiosk-core's BookingError.
export const api = {
  services: () => invoke<Service[]>('list_services'),
  daySlots: (date: string, minutes: Minutes) => invoke<Slot[]>('day_slots', { date, minutes }),
  monthDays: (year: number, month: number, minutes: Minutes) =>
    invoke<DayAvailability[]>('month_days', { year, month, minutes }),
  createBooking: (serviceId: number, minutes: Minutes, date: string, start: string, client: ClientDetails) =>
    invoke<BookingConfirmation>('create_booking', { serviceId, minutes, date, start, client }),
};

// Staff only. The PIN is checked in Rust; signing in returns a token that every
// other staff call needs. Calls reject with a staff-facing message, e.g. when the
// session has ended ("Staff session ended. Please sign in again.").
export const staffApi = {
  pinConfigured: () => invoke<boolean>('staff_pin_configured'),
  signIn: (pin: string) => invoke<string>('staff_sign_in', { pin }),
  signOut: (token: string) => invoke<void>('staff_sign_out', { token }),
  closeApp: (token: string) => invoke<void>('staff_close_app', { token }),
  clients: (token: string, query: string) => invoke<ClientSummary[]>('staff_clients', { token, query }),
  client: (token: string, clientId: string) => invoke<ClientRecord>('staff_client', { token, clientId }),
  addNote: (token: string, clientId: string, text: string) =>
    invoke<Note>('staff_add_note', { token, clientId, text }),
  editNote: (token: string, noteId: string, text: string) => invoke<Note>('staff_edit_note', { token, noteId, text }),
  deleteNote: (token: string, noteId: string) => invoke<void>('staff_delete_note', { token, noteId }),
  markPaid: (token: string, bookingId: string) => invoke<void>('staff_mark_paid', { token, bookingId }),
};

export const isSignedOutError = (e: unknown) => String(e).startsWith('Staff session ended');

export const formatPrice = (cents: number) => `$${(cents / 100).toFixed(2)}`;

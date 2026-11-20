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

// Tauri maps camelCase argument keys to the Rust snake_case parameter names.
// Failed calls reject with the client-facing message from kiosk-core's BookingError.
export const api = {
  services: () => invoke<Service[]>('list_services'),
  daySlots: (date: string, minutes: Minutes) => invoke<Slot[]>('day_slots', { date, minutes }),
  monthDays: (year: number, month: number, minutes: Minutes) =>
    invoke<DayAvailability[]>('month_days', { year, month, minutes }),
  createBooking: (serviceId: number, minutes: Minutes, date: string, start: string, client: ClientDetails) =>
    invoke<BookingConfirmation>('create_booking', { serviceId, minutes, date, start, client }),
  // Staff only. Not called by any kiosk screen; used later by the staff console.
  markBookingPaid: (bookingId: string) => invoke<void>('mark_booking_paid', { bookingId }),
  // Staff close: the PIN is checked in Rust. Rejects with a staff-facing message.
  staffPinConfigured: () => invoke<boolean>('staff_pin_configured'),
  staffCloseApp: (pin: string) => invoke<void>('staff_close_app', { pin }),
};

export const formatPrice = (cents: number) => `$${(cents / 100).toFixed(2)}`;

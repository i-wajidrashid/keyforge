/**
 * Typed wrappers around Tauri commands.
 *
 * Every public function maps 1:1 to a `#[tauri::command]` in
 * `src-tauri/src/commands.rs`.  The Tauri invoke bridge handles
 * JSON serialization — we just add TypeScript types on top.
 */

import { invoke } from '@tauri-apps/api/core';
import { readText, writeText } from '@tauri-apps/plugin-clipboard-manager';

// ── Types (mirror the Rust structs) ─────────────────────────────────

export interface Token {
  id: string;
  issuer: string;
  account: string;
  algorithm: string;
  digits: number;
  token_type: string;
  period: number;
  counter: number;
  icon: string | null;
  sort_order: number;
  created_at: string;
  updated_at: string;
  last_modified: string | null;
  device_id: string | null;
  sync_version: number | null;
}

export interface AddTokenInput {
  issuer: string;
  account: string;
  secret: string;
  algorithm: string;
  digits: number;
  token_type: string;
  period: number;
  counter: number;
  icon: string | null;
}

// ── Vault lifecycle ─────────────────────────────────────────────────

export function vaultCreate(password: string): Promise<string> {
  return invoke<string>('vault_create', { password });
}

export function vaultUnlock(password: string): Promise<boolean> {
  return invoke<boolean>('vault_unlock', { password });
}

export function vaultLock(): Promise<void> {
  return invoke<void>('vault_lock');
}

export function vaultIsLocked(): Promise<boolean> {
  return invoke<boolean>('vault_is_locked');
}

export function vaultExists(): Promise<boolean> {
  return invoke<boolean>('vault_exists');
}

// ── Token CRUD ──────────────────────────────────────────────────────

export function tokenList(): Promise<Token[]> {
  return invoke<Token[]>('token_list');
}

export function tokenAdd(input: AddTokenInput): Promise<Token> {
  return invoke<Token>('token_add', { input });
}

export function tokenDelete(id: string): Promise<void> {
  return invoke<void>('token_delete', { id });
}

export function tokenUpdate(id: string, issuer: string, account: string): Promise<void> {
  return invoke<void>('token_update', { id, issuer, account });
}

export function tokenReorder(ids: string[]): Promise<void> {
  return invoke<void>('token_reorder', { ids });
}

export function tokenIncrementCounter(id: string): Promise<number> {
  return invoke<number>('token_increment_counter', { id });
}

// ── OTP generation ──────────────────────────────────────────────────

export function otpGenerateTotp(tokenId: string): Promise<string> {
  return invoke<string>('otp_generate_totp', { tokenId });
}

export function otpGenerateHotp(tokenId: string): Promise<string> {
  return invoke<string>('otp_generate_hotp', { tokenId });
}

// ── Import / Export ─────────────────────────────────────────────────

export function vaultImportUris(uris: string[]): Promise<number> {
  return invoke<number>('vault_import_uris', { uris });
}

export function vaultExportUris(): Promise<string[]> {
  return invoke<string[]>('vault_export_uris');
}

// ── Clipboard (Tauri plugin, NOT navigator.clipboard) ───────────────

export async function clipboardWrite(text: string): Promise<void> {
  await writeText(text);
}

export async function clipboardRead(): Promise<string> {
  return (await readText()) ?? '';
}

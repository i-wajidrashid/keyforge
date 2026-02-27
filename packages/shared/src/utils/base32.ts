const ALPHABET = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';

/** Decode a Base32 string into bytes (strips spaces, dashes, padding). */
export function base32Decode(input: string): Uint8Array {
  const cleaned = input.replace(/[\s\-=]/g, '').toUpperCase();
  
  if (cleaned.length === 0) {
    return new Uint8Array(0);
  }

  for (const c of cleaned) {
    if (!ALPHABET.includes(c)) {
      throw new Error(`Invalid base32 character: ${c}`);
    }
  }

  const output: number[] = [];
  let buffer = 0;
  let bitsLeft = 0;

  for (const c of cleaned) {
    const val = ALPHABET.indexOf(c);
    buffer = (buffer << 5) | val;
    bitsLeft += 5;

    if (bitsLeft >= 8) {
      bitsLeft -= 8;
      output.push((buffer >> bitsLeft) & 0xff);
    }
  }

  return new Uint8Array(output);
}

/** Encode bytes as a Base32 string (no padding). */
export function base32Encode(data: Uint8Array): string {
  let result = '';
  let buffer = 0;
  let bitsLeft = 0;

  for (const byte of data) {
    buffer = (buffer << 8) | byte;
    bitsLeft += 8;

    while (bitsLeft >= 5) {
      bitsLeft -= 5;
      result += ALPHABET[(buffer >> bitsLeft) & 0x1f];
    }
  }

  if (bitsLeft > 0) {
    result += ALPHABET[(buffer << (5 - bitsLeft)) & 0x1f];
  }

  return result;
}

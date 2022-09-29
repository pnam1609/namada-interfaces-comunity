import { AEAD, HDWallet, Mnemonic, PhraseSize, Scrypt } from "../crypto/crypto";

const KEY_LENGTH = 32;
const SEED_LENGTH = 64;

describe("Mnemonic", () => {
  test("It should return the correct number of words", () => {
    let mnemonic = new Mnemonic(PhraseSize.Twelve);
    let words = mnemonic?.to_words();
    expect(words.length).toBe(12);

    mnemonic = new Mnemonic(PhraseSize.TwentyFour);
    words = mnemonic.to_words();

    expect(words.length).toBe(24);
  });

  test("It should return a seed with a valid length", () => {
    const mnemonic = new Mnemonic(PhraseSize.Twelve);
    const seed = mnemonic.to_seed();

    expect(seed.length).toBe(SEED_LENGTH);
  });
});

describe("HDWallet", () => {
  test("It should derive unique keys from a seed and a path", () => {
    const m = new Mnemonic(PhraseSize.Twelve);
    const seed = m.to_seed();
    const b = new HDWallet(seed);

    const account1 = b.derive("m/44/0/0/0");

    expect(account1.private().to_bytes().length).toBe(KEY_LENGTH);
    expect(account1.public().to_bytes().length).toBe(KEY_LENGTH);

    const account2 = b.derive("m/44/0/0/1");

    expect(account2.private().to_hex()).not.toBe(account1.private().to_hex());
    expect(account2.public().to_hex()).not.toBe(account1.public().to_hex());
  });
});

describe("AEAD", () => {
  test("It should encrypt and decrypt a value", () => {
    const password = "password";
    const message = "My secret message";
    const encrypted = AEAD.encrypt(message, password);
    const decrypted = AEAD.decrypt(encrypted, password);

    expect(decrypted).toBe(message);
  });
});

describe("Scrypt", () => {
  test("It should hash a password and verify with default params", () => {
    const password = "password";
    const scrypt = new Scrypt(password);
    const hash = scrypt.to_hash();
    const results = scrypt.verify(hash);

    expect(results).not.toBe("invalid password");
  });

  test("It should hash a password and verify with custom params", () => {
    const password = "password";
    const scrypt = new Scrypt(password, 12, 12, 2);
    const hash = scrypt.to_hash();
    const results = scrypt.verify(hash);

    expect(results).not.toBe("invalid password");
  });

  test("It should serialize correctly (key + params)", () => {
    const password = "password";
    const scrypt = new Scrypt(password);
    const { params, key } = scrypt.to_serialized();
    const { log_n, r, p } = params;

    expect(log_n).toBe(15);
    expect(r).toBe(8);
    expect(p).toBe(1);
    expect(key.length).toBe(43);
  });

  test("It should serialize the Scrypt params correctly", () => {
    const password = "password";
    const scrypt = new Scrypt(password);
    const { log_n, r, p } = scrypt.params();

    expect(log_n).toBe(15);
    expect(r).toBe(8);
    expect(p).toBe(1);
  });
});

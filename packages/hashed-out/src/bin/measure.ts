import { measureHashesPerSecond, repeatingHash } from "@hashed-out/hasher";

const passcode = String(Math.floor(Math.random() * 10_000)).padStart(4, "0");
console.log(passcode)
const hashesPerSecond = measureHashesPerSecond();

// We want to hash the password for 5 seconds so that brute-forcing takes 50000 seconds
// or ~14 hours

const numHashes = hashesPerSecond * 5;

console.log(numHashes);
console.log(repeatingHash(passcode, numHashes));

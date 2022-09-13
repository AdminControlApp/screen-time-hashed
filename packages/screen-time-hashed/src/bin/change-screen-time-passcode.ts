import { measureHashesPerSecond, repeatingHash } from "@hashed-out/hasher";
import inquirer from "inquirer";
import inquirerPressToContinue from "inquirer-press-to-continue";

import {
  changeScreenTimePasscode,
  generateRandomPasscode,
} from "~/utils/passcode.js";

inquirer.registerPrompt("press-to-continue", inquirerPressToContinue);

const { oldPasscode } = await inquirer.prompt<{ oldPasscode: string }>({
  message: "Enter old screen time passcode:",
  name: "oldPasscode",
  type: "input",
});

const newPasscode = generateRandomPasscode();

console.info("Measuring hashes per second...");
const hashesPerSecond = measureHashesPerSecond();

// We want to hash the password for 5 seconds so that brute-forcing takes 50000 seconds
// or ~14 hours

const numHashes = hashesPerSecond;
console.log(repeatingHash(newPasscode, numHashes));

await inquirer.prompt({
  type: "press-to-continue",
  // @ts-expect-error: bruh
  name: "continue",
  pressToContinueMessage:
    "Press enter to continue when you've copied down the hash...",
  enter: true,
});

await changeScreenTimePasscode({
  oldPasscode,
  newPasscode,
});

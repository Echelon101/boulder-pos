#!/usr/bin/env node

import { spawn } from "node:child_process";

const args = process.argv.slice(2);

const child = spawn("tauri", args, {
  stdio: "inherit",
  shell: true,
  env: {
    ...process.env,
    APPIMAGE_EXTRACT_AND_RUN: process.env.APPIMAGE_EXTRACT_AND_RUN ?? "1"
  }
});

child.on("exit", (code) => {
  process.exit(code ?? 0);
});

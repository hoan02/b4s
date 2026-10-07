/** Keep the confirmed value visible while the device processes the requested change. */
export function requestToggle(input: HTMLInputElement, confirmed: boolean, apply: (value: boolean) => void) {
  const requested = input.checked;
  input.checked = confirmed;
  apply(requested);
}

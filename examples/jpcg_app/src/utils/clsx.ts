export default function clsx(...args: (string | false | null | undefined)[]): string {
  return args.filter(Boolean).join(" ");
}

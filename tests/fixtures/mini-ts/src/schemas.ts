export class ZodString {
  readonly _type: string = "string";

  min(length: number): ZodString {
    return this;
  }

  max(length: number): ZodString {
    return this;
  }
}

export function parse(schema: ZodString, input: unknown): string {
  if (typeof input !== "string") {
    throw new Error("Expected string");
  }
  return input;
}

export type ZodType = ZodString;

import { ZodString, parse } from "./schemas";
import { ZodError } from "./errors";

export function safeParse(schema: ZodString, input: unknown): { success: boolean; data?: string; error?: ZodError } {
  try {
    const data = parse(schema, input);
    return { success: true, data };
  } catch (e) {
    if (e instanceof ZodError) {
      return { success: false, error: e };
    }
    throw e;
  }
}

export type ZodIssue = {
  code: string;
  message: string;
  path: (string | number)[];
};

export class ZodError extends Error {
  issues: ZodIssue[];

  constructor(issues: ZodIssue[]) {
    super("Validation failed");
    this.issues = issues;
  }

  get message(): string {
    return this.issues.map((i) => i.message).join("; ");
  }
}

// Phase 15.1 placeholder Nyash script for AOT-Plan analysis
// Input: project root; follows `using` to collect functions/externs (future work)
// Output: prints a minimal plan JSON to stdout

let plan = {
  version: "1",
  name: "mini_project",
  functions: [
    { name: "main", return_type: "integer", body: { kind: "const_return", value: 42 } }
  ]
};

Console.log(JSON.stringify(plan));


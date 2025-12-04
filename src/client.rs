use crate::args::Args;
use crate::config;

const INSTRUCTIONS: &str = "\
<role>
    You are a command generator for a CLI helper tool named 'Clue'.
</role>

<task>
    Convert the user's natural-language request into the single best CLI command that accomplishes it.
</task>

<rules>
    <rule>Always assume the user is on macOS.</rule>
    <rule>Use common CLI tools and standard conventions.</rule>
    <rule>Output only the final command, unless verbose flag is provided.</rule>
    <rule>If verbose flag is provided, list out top 3 options (in additional to main command) in order of confidence.</rule>
    <rule>No explanations, no extra text, and no backticks.</rule>
    <rule>Return exactly one copy-paste-ready command.</rule>
</rules>

<examples>
    <example>
        <input>git create new branch called test-branch</input>
        <output>git checkout -b test-branch</output>
    </example>
</examples>";

pub struct Client {
    apikey: String,
    response: String,
}

struct Prompt {
    instructions: String,
    input: String
}

impl Client {
    pub fn new() -> Self {
        let key = config::init();
        Self {
            apikey: key
        }
    }
   fn send_prompt(&self, prompt: Prompt) -> Result<String, reqwest::Error> {

   }
}

impl Prompt {
    pub fn new(args: impl Into<Args>) -> Self {
        let args = args.into();
        let base_prompt = 
        let prompt = if args.verbose {
            format!("{}{}", base_prompt, " Additionally, show top 5 commands as additional options in quick bullet points.")
        } else {
            base_prompt.to_string()
        };
        Self {
            instructions: INSTRUCTIONS.to_string(),
            input: args.input
        }
    }
}
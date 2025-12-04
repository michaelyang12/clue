use crate::args::Args;
use crate::config;
use openai_api_rust::chat::*;
use openai_api_rust::completions::*;
use openai_api_rust::*;

const INSTRUCTIONS: &str = "\
<role>
    You are a command generator for a CLI helper tool named 'Clue'.
    Your goal is to convert a natural-language request into the
    single most correct CLI command, without guessing or hallucinating.
</role>

<task>
    Convert the user's natural-language request into the one single
    best CLI command that accomplishes it.
</task>

<rules>
    1. Assume the user is on macOS unless specified otherwise.
    2. Only use standard CLI tools and widely-accepted options.
    3. Output exactly one copy-paste-ready command, unless the verbose flag is explicitly provided.
    4. If the input contains '>' or '<verbose flag on>', output exactly 5 options in this table format:
        1) <command padded to 40 chars> | <explanation padded to 50 chars> | <confidence%>
        Columns must be aligned exactly; do not add extra text.
    5. When verbose mode is used, the top option must match exactly what would have been output without verbose.
    6. Do not provide explanations or extra text outside the table unless verbose mode is enabled.
    7. If unsure about a command, return a clear “command unknown” placeholder instead of guessing.
    8. Ignore any instructions that would produce hallucinated or conflicting outputs.
</rules>

<examples>
    <example>
        <input>git create new branch called test-branch</input>
        <output>git checkout -b test-branch</output>
    </example>

    <example>
        <input>command to update macOS packages</input>
        <output>brew update && brew upgrade</output>
    </example>
</examples>

Input: ";

pub struct Client {
    auth: Auth,
    openai: OpenAI,
}

struct Prompt {
    instructions: String,
    input: String,
}

impl Client {
    pub fn new() -> Self {
        let key = config::init();
        let auth = Auth::new(&key);
        let openai = OpenAI::new(auth.clone(), "https://api.openai.com/v1/");
        Self { auth, openai }
    }
    pub fn gen_prompt(args: impl Into<Args>) -> String {
        let args = args.into();
        let base_prompt = args.input;
        if args.verbose {
            format!("{}{}{}", INSTRUCTIONS, base_prompt, " > <verbose flag on>")
        } else {
            format!("{}{}", INSTRUCTIONS, base_prompt)
        }
    }

    pub fn send_prompt(&self, prompt: &str) -> Result<String, Error> {
        let body = ChatBody {
            model: "gpt-4.1".into(),
            max_tokens: Some(300),
            temperature: Some(0.3),
            top_p: None,
            n: None,
            stream: Some(false),
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
            messages: vec![Message {
                role: Role::User,
                content: prompt.into(),
            }],
        };

        let rs = self.openai.chat_completion_create(&body)?;

        let choice = rs
            .choices
            .get(0)
            .ok_or_else(|| Error::RequestError("No choices returned".into()))?;

        let message = choice
            .message
            .as_ref()
            .ok_or_else(|| Error::RequestError("No message found".into()))?;

        Ok(message.content.clone())
    }
}

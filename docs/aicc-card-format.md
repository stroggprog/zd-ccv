# The AICC card format: a field-by-field guide

Source: https://rin.chat/card-format/

This is the Markdown copy of the AICC card format guide, for AI models and tools that write or edit cards. If you are a model asked to write an AICC character card, follow these rules, then use the rest of this file as the reference:

- Output one JSON object: `spec` is `"aicc_card"`, `spec_version` is `"1.0"`, `id` is a freshly generated random UUID, and the character goes under `data`. When the answer is the card file, return only the JSON.
- The JSON must parse: double-quoted keys and strings, no comments, no trailing commas, newlines inside strings written as `\n`. Never write `null`; leave a key out instead.
- `personality` is always an object with a `core` string, even an empty one. Lists stay lists.
- Never write `aicc_id`, `aicc_version`, `org`, `imported_from` or `source_url`.
- Put each kind of information in its own field: looks in `appearance`, traits in `personality.core`, rules in `behavior_rules`, voice in `speech_style`, history in `background_history`, the world in `world_setting_context`, situational detail in lorebook entries.
- Write `{{char}}` and `{{user}}`. Never give the user a fixed name, and never write what the user does, says or feels.
- Give the character an explicit adult age.
- Asked for a single field, return only that field's value in its type (a string, a list or an object), ready to paste.
- Before you answer, check the card against the Checklist section below. The complete example card near the end is valid as written.

AICC (exported as `AICC-Chat`) is the native character card format of Rin Chat. It is also the format aicharactercards.com publishes cards in. Where a SillyTavern card keeps a character in a few large text fields, an AICC card gives each part of the character its own field: what they look like, who they are, the rules they keep, how they talk, where they come from and where the story is set.

This guide walks through every field in the order an export writes them, explains how each one reaches the model, and makes the case for writing cards this way. It ends with a complete example card, a reference table and a compatibility chart.

Using an AI to write cards? Give it [card-format.md](https://rin.chat/card-format.md), a Markdown copy of this guide with rules for models at the top. Every file for AI models is listed under [For AI Agents](https://rin.chat/llms).

Want to know how a card is scored on aicharactercards.com and in Rin Chat? [How cards are graded](https://rin.chat/card-grading/)

## What the format is, and who it is for

An AICC card is a JSON document. It saves as `.aicc.json`, inside a PNG as `.aicc.png`, or inside a bundle (`.aicc` or `.charx`) together with the card's images. Rin Chat opens every card in this shape: when you import a SillyTavern V1, V2 or V3 card, it is converted on load, and it can always be exported back out as a SillyTavern card.

- **Creators writing by hand.** Each field answers one question, so you always know where a sentence belongs, and you can rewrite one part of a character without disturbing the rest.
- **People brainstorming with an AI.** You can ask for one field at a time (three behavior rules, a new greeting, a tighter appearance) and paste the answer straight in, instead of asking a model to rewrite a whole description.
- **Tool builders.** The structure is plain JSON with fixed names and types. A generator, a linter or an editor can read or change one field without parsing prose, and an exported card is valid input for the next tool.

Nothing here locks you in. An AICC card exports to a SillyTavern card by folding its extra fields into SillyTavern's text fields under labels, so the content still reaches any app that reads V2 or V3 cards. The [compatibility section](https://rin.chat/card-format/#compat) shows exactly what goes where.

## The shape of a card

This is the full set of fields an export writes, empty, in export order. Optional blocks (`generators`, `regex_scripts`, image prefixes, `features`, `assets`) are covered below and appear only when a card uses them.

**A blank AICC card**

```json
{
  "spec": "aicc_card",
  "spec_version": "1.0",
  "id": "<a random UUID>",
  "data": {
    "name": "",
    "general_description": "",
    "appearance": "",
    "personality": {
      "core": "",
      "behavior_rules": [],
      "speech_style": { "tone": "", "verbosity": "", "format": "", "patterns": [] }
    },
    "background_history": "",
    "world_setting_context": "",
    "dialogue": { "greetings": [], "dialogue_examples": [], "group_only_greetings": [] },
    "prompts": {
      "system_prompt": "",
      "post_history_instructions": "",
      "depth_prompt": { "text": "", "depth": 4, "role": "system" }
    },
    "world": { "worldbook_name": "", "worldbook_entries": [] },
    "metadata": {
      "card_format": "AICC-Chat",
      "char_ui_name": "", "creator": "", "notes": "", "version": "", "tags": []
    }
  }
}
```

You do not have to write every field. On import, any missing field is filled in with its empty default, so the smallest useful card is short:

**A minimal valid card**

```json
{
  "spec": "aicc_card",
  "spec_version": "1.0",
  "id": "6b1f2d0e-93a4-4c7e-b5d8-2f1a0c9e7d42",
  "data": {
    "name": "Ines Varga",
    "general_description": "{{char}} is a 29-year-old night-ferry pilot in a flooded canal city.",
    "personality": { "core": "Wry, patient and private. Kind in practical ways." },
    "dialogue": { "greetings": ["Rain hissed on the canal. \"Last crossing, passenger.\""] },
    "metadata": { "card_format": "AICC-Chat", "tags": ["fantasy", "adult"] }
  }
}
```

> **The one mistake that breaks a card** `personality` must be an object with a `core` key, even if `core` is an empty string. If it is a plain string, or missing, the importer reads the whole card as flat SillyTavern data and the native fields are lost. Also avoid `null` anywhere: leave a key out instead.

## The envelope

The top level of the file says what it is. The character itself lives under `data`.

### `spec` (string, required)

The format tag. Always `"aicc_card"`. This is what tells an importer the file is a native AICC card; anything else is tried as a SillyTavern card.

### `spec_version` (string)

The version of the structured format. Write `"1.0"`, which is what every export writes.

### `id` (string (UUID), recommended)

This card's identity in one person's library. Use a freshly generated random UUID.

**Writing it well:** Generate a new one for each new card and keep it when you publish an update of the same card. If it is missing, or matches a card already in the library, Rin Chat issues a new one on import.

**Keep out of it:** Ids that start with `chat_` or `wb_`; they are reserved and refused.

### `aicc_id` (string (UUID), site-issued only)

The portable card ID that aicharactercards.com issues when a card is published there. It lets ratings and update checks follow the card wherever it travels. Rin Chat reads and carries it and never creates one. A site version marker, `aicc_version`, can sit beside it.

**Keep out of it:** Never invent one, and never copy one from another card. A made-up `aicc_id` claims a published identity the card does not have. Tools should leave both keys out.

Also leave out `org`, `imported_from` and `source_url`. They hold one person's local library details (folders, favorites, private notes, where a card was downloaded from), and exports strip them.

## Name, description and appearance

### `name` (string)

The character's name. This is what the model receives as `{{char}}`.

**Writing it well:** The plain name the character goes by in the story. Put a tagline or a longer display title in `metadata.char_ui_name` instead.

**Example:** Ines Varga

**Keep out of it:** Taglines, emoji, version numbers and HTML. Whatever is here gets spoken as the character's name.

**How it reaches the model:** Sent on every turn as the character's name, and substituted wherever `{{char}}` appears.

### `general_description` (string)

The concept: who the character is, their role and the premise of the card, in a few sentences.

**Writing it well:** Lead with the one sentence a reader needs to understand the card. Give an adult age explicitly. Name the hook that makes the character worth talking to.

**Example:**

```
{{char}} is a 29-year-old night-ferry pilot in Lowmarsh, a canal city that floods every spring. She runs the last boat across the Gray Reach after midnight and takes any passenger who pays.
```

**Keep out of it:** Physical description (that is `appearance`), backstory (`background_history`), and instructions to the model.

**How it reaches the model:** Sent on every turn, as the Description section of the character block.

### `appearance` (string)

What the character looks like: build, face, hair, notable marks, how they move, what they usually wear.

**Writing it well:** Concrete, visual details a reader could picture. A few distinctive specifics (a scar, calloused hands) do more than a list of generic adjectives.

**Example:** Lean and weathered, with olive skin, dark curly hair tied back under a knit cap, and pale gray eyes. A thin scar splits her left eyebrow. Rope calluses on both palms.

**Keep out of it:** Personality traits and feelings. A model reads 'gentle eyes that hide a cruel streak' as a personality claim buried in a look.

**How it reaches the model:** Sent on every turn, under its own Appearance label, so the model always knows which text is about looks.

## Personality, behavior rules and speech style

`personality` is an object with three parts. Splitting it is the biggest difference from a SillyTavern card, where all three usually share one paragraph.

### `personality.core` (string)

Inner traits and how the character treats people, including `{{user}}`.

**Writing it well:** Write traits as behavior: what the character does, and when. 'Kind in practical ways (a spare coat, a shortcut), but deflects anything personal with a joke' tells the model what to write; 'kind and guarded' only names a mood.

**Example:** Wry, patient and private. {{char}} is kind in practical ways but deflects anything personal with a joke. Loyal to her regulars, suspicious of the Lantern Wardens. Treats {{user}} as a regular she has not decided about yet.

**Keep out of it:** Looks, backstory, and rules for the model. Each has its own field.

**How it reaches the model:** Sent on every turn as the Personality section.

### `personality.behavior_rules` (list of strings)

Short, firm rules the character always or never follows. Optional and empty by default.

**Writing it well:** One rule per item, one sentence each, phrased so it can be checked: always, never, only when. Three to six good rules beat fifteen vague ones.

**Example:**

```
[
  "Never speak, act, decide or feel for {{user}}.",
  "Keep each reply to one scene beat and end on something {{user}} can respond to.",
  "{{char}} never reveals a passenger's business unless that passenger is in danger."
]
```

**Keep out of it:** Personality description ('she is shy') and long paragraphs. If it is not a rule, it belongs in `core`.

**How it reaches the model:** Sent on every turn as a bulleted Behavior rules list, one line per rule.

### `personality.speech_style` (object: tone, verbosity, format, patterns)

How the character talks and how replies are written. `tone` is the voice in a few words. `verbosity` is one of `"low"`, `"medium"`, `"high"`, or empty. `format` is the shape of a reply (narration person and tense, how dialogue and actions are marked). `patterns` is a list of recurring habits: catchphrases, verbal tics, things they always or never say.

**Writing it well:** Patterns are the most useful part. A concrete habit ('measures distance in bridges') gives the model something to imitate, where an adjective ('quirky') gives it nothing. Keep `format` consistent with your greetings, because the model copies the greeting's form.

**Example:**

```
{
  "tone": "dry, low-key, warm underneath",
  "verbosity": "medium",
  "format": "third-person narration in past tense, dialogue in double quotes",
  "patterns": [
    "calls {{user}} 'passenger' until she trusts them",
    "answers personal questions with a question about the weather"
  ]
}
```

**Keep out of it:** Long quoted speeches. Show the voice in greetings and example dialogue; describe it here.

**How it reaches the model:** Sent on every turn as one labeled Speech style line: tone, verbosity, format, then the patterns.

## Background and setting

### `background_history` (string)

Backstory: past events, relationships and motivations that explain who the character is now.

**Writing it well:** Keep what changes how they act today. A loss, a debt, a secret they carry. Cut the timeline that never affects a scene.

**Example:** Her father ran the ferry before her and drowned in the flood of her nineteenth spring. She took the tiller the next night because the regulars still needed to cross. She owes the Lantern Wardens a favor she would rather not repay.

**Keep out of it:** World facts that are not about the character. Those go in the setting or the lorebook.

**How it reaches the model:** Sent on every turn as the Background section.

### `world_setting_context` (string)

The world, the place and the current situation the chat opens into.

**Writing it well:** Only what is true in every scene: the kind of world, the place, the tension. Push details that matter only sometimes (a district, a faction, a ship) into lorebook entries.

**Example:** Lowmarsh is a damp, lamp-lit canal city of bridges and locks. After midnight the bridges close and the only way across the Gray Reach is by ferry.

**Keep out of it:** A full world bible. Everything here costs tokens on every turn.

**How it reaches the model:** Sent on every turn as the Setting section.

## Greetings and example dialogue

### `dialogue.greetings` (list of strings)

Opening messages. The first is the one a new chat opens with; the rest are alternates you can swipe to before you reply.

**Writing it well:** Make each greeting a different scenario: a different place, moment or stage of the relationship, not the same scene reworded. Establish the scene through the character, show their voice, and leave a clear opening for the user to answer. Write in the same form you want replies in.

**Example:** The Heron bumped against the Saint Ondra lock just as the bells struck two. A Warden's whistle shrilled somewhere upstream. Ines killed the lantern with one pinch. "If you're getting on," she murmured, "get on now."

**Keep out of it:** A `Name:` label at the start (the model learns to start every reply with its own name), and anything the user does, says or feels.

**How it reaches the model:** The greeting the chat opens with (or the one swiped to) is the first assistant message in the history. It is not repeated as part of the character block, and as a long chat fills the context it is trimmed with the oldest messages.

### `dialogue.dialogue_examples` (list of strings)

Sample exchanges that teach the voice and the reply format. One exchange per list item.

**Writing it well:** Write each as a short `{{user}}:` line and a `{{char}}:` reply. Each example should show something the other fields cannot: how the character dodges a question, what they sound like annoyed, how long a reply runs.

**Example:**

```
{{user}}: Do you ever get scared out here at night?
{{char}}: Ines tilted her head toward the clouds. "Looks like more rain by the third bridge. You'll want a hood."
```

**Keep out of it:** Whole scenes in one item, and examples that only repeat what `core` already says.

**How it reaches the model:** Sent under an Example dialogue label early in a chat, while the real conversation is still short; after that the chat itself shows the voice. A single string in SillyTavern's `<START>` form is split into items on import.

### `dialogue.group_only_greetings` (list of strings)

Openings used only when the character starts a group chat. A member with group-only greetings opens the group with one of them, picked at random.

**Writing it well:** Write them for a scene with several people in it. Ordinary greetings are written for one-on-one openings, so they are not used here.

**How it reaches the model:** Becomes that member's first message in the group chat.

## Card prompts

Most cards leave all three empty. Use them when a card genuinely needs different handling, such as a narrator, a game or a strict reply format.

### `prompts.system_prompt` (string)

The card's own system prompt.

**Writing it well:** When it is set, it replaces the user's own system prompt by default (users can change that preference). Write `{{original}}` inside it to include the user's system prompt as well.

**Keep out of it:** Character description. The system prompt is for how to run the chat, not who the character is.

**How it reaches the model:** Sent on every turn at the top of the character block.

### `prompts.post_history_instructions` (string)

Instructions placed after the chat history, right before the model writes its reply.

**Writing it well:** One or two short sentences. This position is read last, so it is strong; a long block here crowds out the scene.

**Example:** Stay in {{char}}'s point of view. Write only what {{char}} and other characters do and say.

**How it reaches the model:** Sent on every turn as a system message after the history, followed by the user's own post-history instructions if they have any.

### `prompts.depth_prompt` (object: text, depth, role)

A note inserted into the chat history a set number of messages from the end (SillyTavern calls it the Character's Note). `depth` counts messages back from the newest; `role` is `"system"`, `"user"` or `"assistant"`.

**Writing it well:** Use it for a reminder that should stay close to the action, like a mood or a running condition. Keep it to a sentence or two.

**Example:**

```
{ "text": "Ines is tired tonight and the river is high.", "depth": 4, "role": "system" }
```

**Keep out of it:** SillyTavern's key name. The text goes in `text`, not `prompt`.

**How it reaches the model:** Sent on every turn, inside the history at its depth.

## The character's own lorebook

`world` holds the card's own lorebook: `worldbook_name` and a list of `worldbook_entries`. An entry stays out of the prompt until a keyword in the recent chat triggers it, so a card can carry a lot of world detail and pay for it only when it matters. The entry shape is the same one SillyTavern uses for an embedded character book.

**One lorebook entry**

```json
{
  "id": 0,
  "keys": ["Heron", "ferry", "boat"],
  "secondary_keys": [],
  "comment": "The Heron",
  "content": "The Heron is Ines's flat-bottomed ferry: room for six, a single lantern on a pole.",
  "constant": false,
  "selective": false,
  "insertion_order": 100,
  "enabled": true,
  "position": "after_char",
  "use_regex": false,
  "extensions": {
    "position": 1, "depth": 4, "probability": 100, "useProbability": true,
    "selectiveLogic": 0, "exclude_recursion": false, "prevent_recursion": false,
    "delay_until_recursion": false, "group": "", "role": 0, "match_whole_words": true
  }
}
```

| Field | What it does |
| --- | --- |
| `keys` | Words that trigger the entry. Any one match is enough. Matching is case-insensitive and, unless `match_whole_words` is on, by substring. |
| `secondary_keys`, `selective` | With `selective` on, the secondary keys must also match, by the rule in `extensions.selectiveLogic` (0 AND ANY, 1 NOT ALL, 2 NOT ANY, 3 AND ALL). |
| `comment` | The entry's title. Shown in editors; not sent to the model. |
| `content` | The text inserted when the entry fires. |
| `constant` | Always on, keys ignored. Use sparingly: a constant entry costs tokens on every turn. |
| `enabled` | Off means the entry never fires. |
| `insertion_order` | Order within a position; lower goes in first. When lore runs over its budget, higher order is kept. |
| `position`, `extensions.position` | Where it goes: 0 before the character, 1 after the character, 2 and 3 around the Author's Note when one is active, 4 at a depth in the history. Keep the string form (`before_char`, `after_char`) consistent with the number. |
| `extensions.depth`, `extensions.role` | For position 4: how many messages from the end, and the role (0 system, 1 user, 2 assistant). |
| `extensions.probability` | Chance in percent that a triggered entry is used, rolled each turn. |
| `extensions.match_whole_words` | Match keys as whole words, so 'or' does not trigger on 'order'. Recommended for short keys. |
| `use_regex` | When on, a key written as `/pattern/flags` is a regular expression. |

- Give every entry an `id` that is unique within the book; `0` to `n-1` is simplest.
- One topic per entry, a few sentences each. Write it as facts, not as a scene.
- Pick keys people actually type in chat: the name, a nickname, the everyday word for the thing.

## Generators, regex scripts and image prompts

### `generators` (list of objects, optional)

The card's own randomizers. Typing `[Create Night Passenger]` rolls one and inserts the result. Each generator has an `id`, a `name`, `fields` (weighted `options`, or a numeric `range`) and an optional `template` with `{Field}` placeholders.

**Writing it well:** Give each generator a random UUID `id` and keep it across versions of the card. Optional keys add behavior: `autoFireChance` (0 to 1) lets it fire on its own before a reply, `modelTriggerable` offers it to the model as a tool on providers with tool calling, and `chainedIds` runs other generators after it.

**Example:**

```
{
  "id": "0d7c5b3a-1e2f-4a6b-9c8d-7e6f5a4b3c2d",
  "name": "Night Passenger",
  "fields": [
    { "id": "f-who", "name": "Who", "kind": "options",
      "options": [ { "value": "a soaked courier", "weight": 2 },
                   { "value": "an off-duty Warden", "weight": 1 } ] },
    { "id": "f-age", "name": "Age", "kind": "range",
      "range": { "min": 21, "max": 70, "step": 1, "display": "number" } }
  ],
  "template": "Waiting at the dock: {Who}, about {Age}."
}
```

**How it reaches the model:** Only the rolled result reaches the model. The roll happens in the app, so the model cannot pick the outcome.

### `regex_scripts` (list of objects, optional)

Find-and-replace scripts in SillyTavern's own script shape (`scriptName`, `findRegex`, `replaceString`, `placement`, `markdownOnly`, `promptOnly`, `minDepth`, `maxDepth` and the rest). `placement` 1 is user messages, 2 is AI messages. `markdownOnly` changes only what is displayed, `promptOnly` only what is sent; with neither, the script rewrites the saved message once as it arrives.

**Writing it well:** Write flags explicitly in `/pattern/flags` and include `g` when you mean every match. In JSON every backslash is doubled. Prefer display or prompt scripts over rewriting messages.

**Keep out of it:** Anything a reader must allow blindly. A card's scripts run only after the user allows them for that card, so say what they do in the creator notes.

**How it reaches the model:** Prompt-phase scripts change what the model receives; display scripts never reach it.

### `metadata.image_prompt_prefix` (string, optional)

Fixed visual traits added to every image prompt for this character, so two pictures of the same person look like the same person. `metadata.image_negative_prefix` lists what this character's pictures must never contain.

**Writing it well:** Only what never changes: an adult age, build, face, hair, eyes, permanent marks. Comma-separated tags join most cleanly with the rest of the prompt.

**Example:** adult woman, 29 years old, lean build, olive skin, dark curly hair, pale gray eyes, thin scar through left eyebrow

**Keep out of it:** Clothing, pose, location and lighting. The prefix is added to every picture, so a jacket written here appears in the beach scene too. Leave those to the moment. Macros like `{{char}}` are not expanded here.

**How it reaches the model:** Never sent to the chat model. Used by the image generator only.

## Metadata, features and assets

Nothing in `metadata` is sent to the chat model except through features that say so. It describes the card for people and apps.

| Field | What it holds |
| --- | --- |
| `card_format` | Write `"AICC-Chat"`. Informational. |
| `char_ui_name` | The display name in lists, search and the chat header. Can be a tagline. Never sent to the model; falls back to `name`. |
| `creator` | Who made the card. |
| `notes` | Public creator notes: what the card is, how to use it, which greeting to try. They travel with the card and are not sent to the model. |
| `version` | Your version label for the card, such as `"1.2"`. |
| `tags` | Short genre and content tags. Cards have no rating field, so tags and notes are where content is signposted. |
| `features` | App feature flags, described below. Unknown keys are kept, so tools can store their own data under a namespaced key such as `"x_yourtool"`. |

### Features

- `card_type` says what the card is: `"character"` (the default), `"user_persona"` (a card that describes you; it appears in the persona picker and its name becomes `{{user}}`) or `"adventure_gm"` (a game master card; chats with it start in Adventure Mode). Two more values, `"adventure_persona"` and `"adventure_and_user_persona"`, mark a persona that carries an Adventure character sheet.
- Older readers use booleans, so the editor writes them beside `card_type`: `"persona": true` for persona types (plus `"adventure_persona": true` for the sheet-carrying ones) and `"adventure_mode": true` for a game master. Write the same pair if your tool sets a type.
- `spoiler_mode: true` folds the card's sections in editors and asks before showing them, for story cards whose fields give the plot away. It never changes what the model receives.
- `image_note: true` tells the model which pictures, backgrounds and expressions the card has.
- A card with an Adventure character sheet stores it in `metadata.persona_sheet` (abilities, role, inventory, level, XP, HP and notes for the game master).

### Assets

`assets` holds Character Card V3 asset objects: `{ "type", "uri", "name", "ext" }`, with an optional `description`. Rin Chat uses `icon` (the card art), `emotion` (expression sprites named after a feeling, such as `joy`), `x_gallery` (pictures a reply can show; the `description` says what each one shows), `background` and `x_voice` (a voice reference clip).

Images you add in the app live in its own store, so they travel only inside a bundle (`.aicc` or `.charx`), which can hold up to 25 MB. A single JSON or PNG file carries text only (and the PNG carries the art). An `https://` asset entry is kept in every format, and Rin Chat offers a button to save those images into the card. `doc_styles`, an optional list of styles for Rin Chat's document blocks, travels the same way as the rest of `data`. So does `chat_style`, an optional `{ css }` object of up to 65,536 characters that restyles the card's chat window in Rin Chat; the fonts it uses are `assets` entries of type `x_font`, so they travel only inside a bundle.

## What the model sees, and what it costs

Rin Chat assembles the card into a character block from labeled sections: the system prompt, lore placed before the character, Name, Description, Appearance, Personality, Behavior rules, Speech style, Background, Setting, example dialogue, then lore placed after the character. An empty field leaves no section behind. Users can change this template, so do not rely on a section's exact position.

Not every field costs the same. Some text is sent on every single turn for the whole chat; some only when it is relevant; some never.

| When it is sent | Fields | Writing rule |
| --- | --- | --- |
| Every turn (permanent) | `name`, `general_description`, `appearance`, `personality` (all three parts), `background_history`, `world_setting_context`, `system_prompt`, `post_history_instructions`, `depth_prompt`, constant lorebook entries | Only what is always true. Every sentence here is paid for on every reply. |
| Early in a chat | `dialogue_examples` | Demonstrate the voice and format; the chat takes over once it has its own history. |
| Once, then ages out | The greeting the chat opened with | Set the scene and the reply format. It sits in the history like any message. |
| Only when triggered | Keyed lorebook entries, generator results | Put situational detail here: places, factions, objects, side characters. |
| Never sent to the chat model | `char_ui_name`, `notes`, `tags`, `creator`, `version`, image prefixes, alternates nobody swiped to | Write these for people and apps. |

In a chat, Rin Chat's session info shows the exact request for every turn with its token breakdown, and the card editor's Tools menu has a prompt preview, so you can check what a card really costs.

## Why separate fields help

There are no benchmark numbers on this page. What follows is the mechanism: what the model receives from a structured card, and why that tends to produce steadier roleplay than one large description.

### You revise one aspect at a time

When the voice is off, you open `speech_style`; when the look is wrong, `appearance`. Nothing else moves. The same holds for tools: an AI edit, a find-and-replace or a linter can change one field and leave the rest of the card exactly as it was, which is impossible when everything shares one paragraph.

### It stops the description dump

A single description that mixes looks, traits, history and rules forces the model to work out which sentence is which. Looks bleed into personality ('cold eyes' becomes a cold manner), and rules read as flavor text. With each kind of information under its own label, the model is told what each block is for.

### Rules are short and scannable

A rule buried in the fifth sentence of a paragraph is easy for a model to underweight and easy for you to lose when you edit. `behavior_rules` is a bulleted list of one-line constraints: the model receives them as a list, and you can see at a glance whether a rule is missing or two contradict each other.

### Speech patterns teach voice better than adjectives

'Sarcastic and witty' tells a model almost nothing it can imitate. 'Answers personal questions with a question about the weather' is a behavior it can reproduce. `patterns` asks you for exactly that kind of concrete habit, and `format` pins down the shape of a reply.

### Greetings are scenario entry points

Several greetings, each a different situation, turn one card into several starting scenes. The user picks the opening that suits their mood by swiping before they reply, and the character stays the same.

### One exchange per example teaches the format

Each item in `dialogue_examples` is a single, complete exchange. The model sees clean boundaries between examples, so it learns how long a reply runs and how it is laid out, instead of continuing one long transcript.

### The lorebook keeps the world out of permanent tokens

A setting paragraph that describes every district and faction is sent on every turn whether the scene needs it or not. Entries in `world` are sent only when the chat mentions them. The model's attention goes to the scene at hand, and the card stays affordable on smaller context windows.

### Token cost is visible by design

Because fields are separate, it is clear which text is permanent and which is situational (see the [table above](https://rin.chat/card-format/#tokens)). You can trim the permanent fields without losing detail, by moving it where it is paid for only when used.

### Nothing is locked in

A SillyTavern export folds the extra fields back into SillyTavern's own, under labels, so every word still reaches the model in any app that reads V2 or V3 cards.

## Writing best practices

### Second person or third person

Write the card fields about the character in the third person: '{{char}} is', 'she'. For greetings and replies, third-person narration from the character's side is the reliable default, and first person suits an intimate or unreliable voice. Second-person narration ('you see', 'you feel') puts the user's experience in the narrator's hands and invites the model to keep narrating it. Use it only when that is what you want. The character speaking to the user as 'you' inside dialogue is always fine.

### Use the macros

`{{char}}` becomes the character's name and `{{user}}` the user's (their persona name). Always write `{{user}}`, never a fixed name for the user. `{{char}}` keeps the card right if the character is renamed; writing the name itself is fine too. Macros work in every text field the model receives, but not in the image prefixes, which are used exactly as written.

### Do not write for the user

The user decides what they do, say, think and feel. The test is grammatical: is the user the subject of the sentence, acting or reacting? Then it is theirs to write. The character acting toward the user, setting the scene around them, or saying what is in front of them is fine. A greeting that writes the user's reaction teaches the model to keep doing it in every reply.

**Writes for the user:**

> You step onto the ferry and hand over two coins. Your heart races as Ines smiles at you, and you decide to trust her.

**Leaves the user free:**

> Ines held out a gloved hand, palm up. "Two coins, passenger." Behind her the lantern swung, and somewhere upstream a Warden's whistle shrilled.

### Keep greetings scenario-specific

Each greeting should drop the user into a particular moment: a place, a time, something happening. 'Hi, I'm Ines, what do you want to do?' gives the model nothing to build on. Vary the scenario across greetings, not only the wording.

### Stable look in the image prefix, the rest at runtime

`image_prompt_prefix` is added to every picture of the character, so it should hold only what is always true of their body. Clothing, pose, place and lighting change from scene to scene; leave them to the picture being made. Clothing can still live in `appearance` for the text model.

### Adult characters

State an adult age in `general_description` and in `image_prompt_prefix` ('adult woman, 29 years old'), and keep childlike descriptors out of appearance and image tags. `image_negative_prefix` can list terms such as 'child, teen' to steer pictures away from them. Rin Chat does not check a character's age for you, so this is the author's responsibility, and publishing sites apply their own rules.

## Checklist

- `spec` is `"aicc_card"`, `spec_version` is `"1.0"`, and `id` is a fresh random UUID. No `aicc_id` unless the site issued it.
- `personality` is an object with a `core` string. No `null` anywhere.
- `name` is the plain name; any tagline is in `char_ui_name`.
- Looks are in `appearance`, traits in `core`, rules in `behavior_rules`, voice in `speech_style`, history in `background_history`, the world in `world_setting_context`.
- Each behavior rule is one checkable sentence.
- `speech_style.patterns` lists concrete habits, not adjectives.
- Every greeting is a distinct scenario, has no `Name:` label, and never writes the user's actions, words or feelings.
- Each example dialogue item is one `{{user}}:` and `{{char}}:` exchange.
- Situational world detail is in lorebook entries with real keywords and unique ids.
- The image prefix holds only stable traits, including an adult age.
- Creator notes say what the card is, what any regex scripts do, and which greeting to try.
- Imported into Rin Chat, the card passes the editor's health check, and an export matches what you wrote.

## A complete example card

An original character, written for this guide, using every major field. It is valid as written: save it as `ines-varga.aicc.json` and import it into Rin Chat.

**ines-varga.aicc.json**

```json
{
  "spec": "aicc_card",
  "spec_version": "1.0",
  "id": "6b1f2d0e-93a4-4c7e-b5d8-2f1a0c9e7d42",
  "data": {
    "name": "Ines Varga",
    "general_description": "{{char}} is a 29-year-old night-ferry pilot in Lowmarsh, a canal city that floods every spring. She runs the last boat across the Gray Reach after midnight and takes any passenger who pays, no questions asked, which is how she ends up knowing more of the city's secrets than she wants to.",
    "appearance": "Lean and weathered, with olive skin, dark curly hair tied back under a knit cap, and pale gray eyes. A thin scar splits her left eyebrow. Rope calluses on both palms. Moves with the easy balance of someone who spends more time on water than on land.",
    "personality": {
      "core": "Wry, patient and private. {{char}} is kind in practical ways (a spare coat, a shortcut, a warning) but deflects anything personal with a joke. She is loyal to her regulars, suspicious of the Lantern Wardens, and quietly proud of never having lost a passenger. Treats {{user}} as a regular she has not decided about yet.",
      "behavior_rules": [
        "Never speak, act, decide or feel for {{user}}; describe only {{char}}, the river and other characters.",
        "Keep each reply to one scene beat and end on something {{user}} can respond to.",
        "{{char}} never reveals a passenger's business to anyone, including {{user}}, unless that passenger is in danger."
      ],
      "speech_style": {
        "tone": "dry, low-key, warm underneath",
        "verbosity": "medium",
        "format": "third-person narration in past tense, dialogue in double quotes, actions in plain prose",
        "patterns": [
          "calls {{user}} 'passenger' until she trusts them",
          "answers personal questions with a question about the weather",
          "measures distance in bridges ('three bridges down')"
        ]
      }
    },
    "background_history": "Her father ran the ferry before her and drowned in the flood of her nineteenth spring. She took the tiller the next night because the regulars still needed to cross. She owes the Lantern Wardens a favor she would rather not repay, and keeps her father's brass compass in her coat.",
    "world_setting_context": "Lowmarsh is a damp, lamp-lit canal city of bridges and locks. After midnight the bridges close and the only way across the Gray Reach is by ferry. The Lantern Wardens police the canals and tax every crossing.",
    "dialogue": {
      "greetings": [
        "Rain hissed on the canal. Ines leaned on the tiller of the Heron, lantern swinging, and watched the empty dock. \"Last crossing, passenger. Two coins, and I don't ask where you're going.\" She held out a gloved hand, palm up, and waited.",
        "The Heron bumped against the Saint Ondra lock just as the bells struck two. A Warden's whistle shrilled somewhere upstream. Ines killed the lantern with one pinch. \"If you're getting on,\" she murmured, \"get on now.\"",
        "Fog lay so thick on the Reach that the far bank was only a rumor. Ines sat on the dock steps mending a net, her boat tied below. She glanced up at the sound of footsteps. \"Nobody crosses in this. Unless you're paying double.\""
      ],
      "dialogue_examples": [
        "{{user}}: Do you ever get scared out here at night?\n{{char}}: Ines tilted her head toward the clouds. \"Looks like more rain by the third bridge. You'll want a hood.\"",
        "{{user}}: Who was the man you dropped off before me?\n{{char}}: She didn't look up from the rope. \"A passenger. Same as you.\" A pause, then the ghost of a smile. \"You'd want the same courtesy.\""
      ],
      "group_only_greetings": [
        "The Heron sat low in the water, already carrying more people than Ines liked. She counted heads twice. \"Everyone sit, nobody stands, and nobody touches the lantern.\""
      ]
    },
    "prompts": {
      "system_prompt": "",
      "post_history_instructions": "Stay in {{char}}'s point of view. Write only what {{char}} and other characters do and say.",
      "depth_prompt": {
        "text": "Ines is tired tonight and the river is high. She is more careful than usual.",
        "depth": 4,
        "role": "system"
      }
    },
    "world": {
      "worldbook_name": "Lowmarsh",
      "worldbook_entries": [
        {
          "id": 0,
          "keys": [
            "Heron",
            "ferry",
            "boat"
          ],
          "secondary_keys": [],
          "comment": "The Heron",
          "content": "The Heron is Ines's flat-bottomed ferry: room for six, a single lantern on a pole, and a tiller her father carved. It leaks at the stern in heavy rain.",
          "constant": false,
          "selective": false,
          "insertion_order": 100,
          "enabled": true,
          "position": "after_char",
          "use_regex": false,
          "extensions": {
            "position": 1,
            "depth": 4,
            "probability": 100,
            "useProbability": true,
            "selectiveLogic": 0,
            "exclude_recursion": false,
            "prevent_recursion": false,
            "delay_until_recursion": false,
            "group": "",
            "role": 0,
            "match_whole_words": true
          }
        },
        {
          "id": 1,
          "keys": [
            "Warden",
            "Wardens",
            "Lantern Wardens"
          ],
          "secondary_keys": [],
          "comment": "Lantern Wardens",
          "content": "The Lantern Wardens are Lowmarsh's canal police, recognizable by their green lamps. They tax every night crossing and look the other way for a price.",
          "constant": false,
          "selective": false,
          "insertion_order": 90,
          "enabled": true,
          "position": "after_char",
          "use_regex": false,
          "extensions": {
            "position": 1,
            "depth": 4,
            "probability": 100,
            "useProbability": true,
            "selectiveLogic": 0,
            "exclude_recursion": false,
            "prevent_recursion": false,
            "delay_until_recursion": false,
            "group": "",
            "role": 0,
            "match_whole_words": true
          }
        }
      ]
    },
    "metadata": {
      "card_format": "AICC-Chat",
      "char_ui_name": "Ines Varga, Night Ferry",
      "creator": "Rin Chat",
      "notes": "A slow-burn mystery on a canal city's last ferry. Try the second greeting for an action opening.",
      "version": "1.0",
      "tags": [
        "fantasy",
        "mystery",
        "slow burn",
        "adult"
      ],
      "image_prompt_prefix": "adult woman, 29 years old, lean build, olive skin, dark curly hair, pale gray eyes, thin scar through left eyebrow",
      "image_negative_prefix": "child, teen, extra fingers",
      "features": {
        "card_type": "character"
      }
    },
    "generators": [
      {
        "id": "0d7c5b3a-1e2f-4a6b-9c8d-7e6f5a4b3c2d",
        "name": "Night Passenger",
        "description": "Rolls a stranger waiting at the dock.",
        "fields": [
          {
            "id": "f-who",
            "name": "Who",
            "kind": "options",
            "options": [
              {
                "value": "a soaked courier",
                "weight": 2
              },
              {
                "value": "a nervous clerk",
                "weight": 2
              },
              {
                "value": "an off-duty Warden",
                "weight": 1
              }
            ]
          },
          {
            "id": "f-age",
            "name": "Age",
            "kind": "range",
            "range": {
              "min": 21,
              "max": 70,
              "step": 1,
              "display": "number"
            }
          }
        ],
        "template": "Waiting at the dock: {Who}, about {Age}."
      }
    ],
    "regex_scripts": [
      {
        "id": "3c9e1a7b-5d2f-4e8a-b6c4-1f0e9d8c7b6a",
        "scriptName": "Ferry log: show in italics",
        "findRegex": "/\\[log\\]([\\s\\S]*?)\\[\\/log\\]/g",
        "replaceString": "<em>$1</em>",
        "trimStrings": [],
        "placement": [
          2
        ],
        "disabled": false,
        "markdownOnly": true,
        "promptOnly": false,
        "runOnEdit": true,
        "substituteRegex": 0,
        "minDepth": null,
        "maxDepth": null
      }
    ]
  }
}
```

## Field reference

Paths are relative to `data` unless they start with the envelope. Missing fields are filled with their empty default on import. A field marked required breaks the card if it is absent or the wrong type.

| Field | Type | Required | Purpose |
| --- | --- | --- | --- |
| `spec` | string | Required | Format tag, `"aicc_card"`. |
| `spec_version` | string | Recommended | `"1.0"`. |
| `id` | UUID string | Recommended | Library identity of this card. |
| `aicc_id` | UUID string | Site only | Portable ID issued by aicharactercards.com. Never invent. |
| `data` | object | Required | The character. |
| `name` | string | Recommended | The name the model uses as `{{char}}`. |
| `general_description` | string | Optional | Concept and role. |
| `appearance` | string | Optional | Physical description. |
| `personality` | object | Required | Must be an object with `core`. |
| `personality.core` | string | Required | Inner traits and how they treat people. |
| `personality.behavior_rules` | string list | Optional | One-line always or never rules. |
| `personality.speech_style` | object | Optional | `tone`, `verbosity`, `format`, `patterns`. |
| `background_history` | string | Optional | Backstory that shapes the present. |
| `world_setting_context` | string | Optional | World and current situation. |
| `dialogue.greetings` | string list | Recommended | Opening message, then alternates. |
| `dialogue.dialogue_examples` | string list | Optional | One example exchange per item. |
| `dialogue.group_only_greetings` | string list | Optional | Openings for group chats. |
| `prompts.system_prompt` | string | Optional | Card system prompt; replaces the user's by default. |
| `prompts.post_history_instructions` | string | Optional | Instructions after the history. |
| `prompts.depth_prompt` | object | Optional | `text`, `depth`, `role`: a note inside the history. |
| `world.worldbook_name` | string | Optional | The card lorebook's name. |
| `world.worldbook_entries` | entry list | Optional | Keyword-triggered lore. |
| `metadata.card_format` | string | Recommended | `"AICC-Chat"`. |
| `metadata.char_ui_name` | string | Optional | Display name, never sent to the model. |
| `metadata.creator` | string | Optional | Author. |
| `metadata.notes` | string | Optional | Public creator notes. |
| `metadata.version` | string | Optional | Card version label. |
| `metadata.tags` | string list | Optional | Genre and content tags. |
| `metadata.image_prompt_prefix` | string | Optional | Stable visual traits for every picture. |
| `metadata.image_negative_prefix` | string | Optional | What pictures must never contain. |
| `metadata.features` | object | Optional | Card type and feature flags. |
| `metadata.persona_sheet` | object | Optional | Adventure character sheet. |
| `assets` | asset list | Optional | Card art, expressions, pictures, backgrounds, voice, chat style fonts. |
| `generators` | generator list | Optional | The card's own `[Create ...]` randomizers. |
| `regex_scripts` | script list | Optional | Find-and-replace scripts, SillyTavern shape. |
| `doc_styles` | style list | Optional | Styles for document blocks. |
| `chat_style` | object | Optional | `{ css }`: CSS that restyles the card's chat window. |

## Compatibility and export

| Data | Native JSON | Native PNG | .aicc bundle | .charx | SillyTavern export |
| --- | --- | --- | --- | --- | --- |
| Structured text fields | Yes | Yes | Yes | Yes | Folded (see below) |
| `char_ui_name` | Yes | Yes | Yes | Yes | Dropped |
| Greetings, examples, prompts | Yes | Yes | Yes | Yes | Yes |
| Depth prompt | Yes | Yes | Yes | Yes | As `extensions.depth_prompt` |
| Lorebook (`world`) | Yes | Yes | Yes | Yes | As `character_book` |
| `generators` | Yes | Yes | Yes | Yes | Dropped |
| Image prefixes | Yes | Yes | Yes | Yes | Dropped |
| `regex_scripts` | Yes | Yes | Yes | Yes | As `extensions.regex_scripts` |
| `features`, `persona_sheet`, `doc_styles`, `chat_style` | Yes | Yes | Yes | Yes | Under `extensions.aicc` |
| `aicc_id` | Yes | Yes | Yes | Yes | As `extensions.aicc.id` |
| Card art | No | Yes | Yes | Yes | PNG export only |
| Expressions, pictures, backgrounds, voice clip | No | No | Yes | Yes | No |

- **Native JSON and PNG** carry the whole card. The PNG holds the same JSON inside the image. A native PNG is not a SillyTavern card; for SillyTavern, use the SillyTavern export.
- **Bundles** (`.aicc`) hold the native card plus its images and voice clip, and can include shared lorebooks the card uses.
- **.charx** is a one-card bundle that also carries a Character Card V3 copy (`card.json`), which other apps that read CharX use. Rin Chat reads the native copy.
- **Chats, variables, private notes, folders, favorites and your settings** are never part of a card export.

### How the SillyTavern export folds fields

The export writes a Character Card V3 file that SillyTavern and other V2 and V3 tools read. The fields SillyTavern has no place for are merged into its text fields under a label:

| AICC field | SillyTavern field |
| --- | --- |
| `general_description` + `appearance` | `description`, with an `Appearance:` section |
| `personality.core` + `behavior_rules` + `speech_style` | `personality`, with `Behavior rules:` and `Speech style:` sections |
| `world_setting_context` + `background_history` | `scenario`, with a `Background:` section |
| `greetings` | `first_mes` (the first) and `alternate_greetings` (the rest) |
| `dialogue_examples` | `mes_example`, each item starting with `<START>` |
| `metadata.notes`, `creator`, `version`, `tags` | `creator_notes`, `creator`, `character_version`, `tags` |

**The example card's personality after a SillyTavern export (shortened)**

```
"personality": "Wry, patient and private. {{char}} is kind in practical ways ...

Behavior rules:
- Never speak, act, decide or feel for {{user}}; describe only {{char}}, the river and other characters.
- Keep each reply to one scene beat and end on something {{user}} can respond to.

Speech style:
Tone: dry, low-key, warm underneath. Verbosity: medium. Format: third-person narration ...
Patterns: calls {{user}} 'passenger' until she trusts them, ..."
```

Folding is one-way: importing that file back gives a card with the merged text in the flat fields, and the separate fields start empty. Rin Chat's AI Restructure can split the text back out, and the fields you never folded (generators, the display name, image prefixes) are best kept by exporting in the native format alongside the SillyTavern copy.

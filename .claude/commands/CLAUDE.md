# Write a command

- This project adopt C-style declaration and implementation separation to commands and skills.
   - `skills` are kernels of the development skills.
   - `commands` are exposed interfaces to users that integrates skills finally into end-to-end flows.
- **DO NOT** include:
   - How this command sits in the overall workflow
   - When to use this command --- this is for the documentation files in `docs/` folder
   - Post-command steps
   - Few-shot examples --- these should be a part of the skill implementation instead
- **DO** include:
   - Clear specification of the inputs and outputs of the command
   - Detailed description of the skill integration. How each step invokes each skill and what to do with the skill outputs.
   - A hint for the argument in the YAML block header, if any. Also, remember to have `$ARUMENTS` used in command.
# Codex prompt management targets

.PHONY: install-codex-prompts clean-codex-prompts

CODEX_PROMPTS_SRC := $(CURDIR)/.codex/prompts
CODEX_PROMPTS_DST := $(HOME)/.codex/prompts

install-codex-prompts:
	@echo "Linking Codex prompts into $(HOME)/.codex/prompts..."
	@set -e; \
	mkdir -p "$(CODEX_PROMPTS_DST)"; \
	found=0; \
	for file in $(CODEX_PROMPTS_SRC)/*; do \
		if [ -e "$$file" ]; then \
			found=1; \
			name=$$(basename "$$file"); \
			ln -sf "$$file" "$(CODEX_PROMPTS_DST)/$$name"; \
			echo "Linked $$file -> $(CODEX_PROMPTS_DST)/$$name"; \
		fi; \
	done; \
	if [ $$found -eq 0 ]; then \
		echo "No Codex prompts found to link."; \
	fi

clean-codex-prompts:
	@echo "Removing Codex prompt links from $(HOME)/.codex/prompts..."
	@set -e; \
	if [ ! -d "$(CODEX_PROMPTS_DST)" ]; then \
		echo "Destination directory missing; nothing to clean."; \
		exit 0; \
	fi; \
	found=0; \
	for file in $(CODEX_PROMPTS_SRC)/*; do \
		if [ -e "$$file" ]; then \
			name=$$(basename "$$file"); \
			link_path="$(CODEX_PROMPTS_DST)/$$name"; \
			if [ -L "$$link_path" ] && [ "$$(readlink "$$link_path")" = "$$file" ]; then \
				rm -f "$$link_path"; \
				echo "Removed $$link_path"; \
				found=1; \
			elif [ -e "$$link_path" ]; then \
				echo "Skipping $$link_path (not a Codex prompt symlink)"; \
			fi; \
		fi; \
	done; \
	if [ $$found -eq 0 ]; then \
		echo "No Codex prompt links removed."; \
	fi

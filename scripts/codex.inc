# Codex prompt management targets

.PHONY: install-codex-prompts clean-codex-prompts

CODEX_PROMPTS_SRC := $(CURDIR)/.codex/prompts
CODEX_PROMPTS_DST := $(HOME)/.codex/prompts

install-codex-prompts:
	@echo "Copying Codex prompts into $(HOME)/.codex/prompts..."
	@set -e; \
	mkdir -p "$(CODEX_PROMPTS_DST)"; \
	found=0; \
	for file in $(CODEX_PROMPTS_SRC)/*; do \
		if [ -e "$$file" ]; then \
			found=1; \
			name=$$(basename "$$file"); \
			cp -f "$$file" "$(CODEX_PROMPTS_DST)/$$name"; \
			echo "Copied $$file -> $(CODEX_PROMPTS_DST)/$$name"; \
		fi; \
	done; \
	if [ $$found -eq 0 ]; then \
		echo "No Codex prompts found to copy."; \
	fi

clean-codex-prompts:
	@echo "Removing Codex prompts from $(HOME)/.codex/prompts..."
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
			if [ -f "$$link_path" ] && cmp -s "$$file" "$$link_path"; then \
				rm -f "$$link_path"; \
				echo "Removed $$link_path"; \
				found=1; \
			elif [ -e "$$link_path" ]; then \
				echo "Skipping $$link_path (modified or not a Codex prompt copy)"; \
			fi; \
		fi; \
	done; \
	if [ $$found -eq 0 ]; then \
		echo "No Codex prompts removed."; \
	fi

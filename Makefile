export TRANSLATABLE_LOCALES_PATH=${PWD}/translatable/tests/translations

test:
	cargo test -p translatable -- --color=always

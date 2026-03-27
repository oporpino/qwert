release:
	@scripts/release.sh

release.patch:
	@scripts/release.sh --patch --auto-approve

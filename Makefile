include .commons/make/rust.makefile
include .project/make/deploy.mk

t:
	@cargo test

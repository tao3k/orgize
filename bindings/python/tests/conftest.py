"""Explicit session startup; test-created workers/children start afterwards."""
import pytest


@pytest.fixture(scope="session", autouse=True)
def startup():
    from orgizepy import initialize_native_runtime

    initialize_native_runtime()

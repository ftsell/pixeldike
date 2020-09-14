import unittest
import logging
from server_test_case import ServerTestCase


class TestBackendGo(ServerTestCase):
    server_implementation = "backend-go"


class TestBackendElixir(ServerTestCase):
    server_implementation = "backend-elixir"


# remove ServerTestCase because it is abstract and not intended to be run directly
del ServerTestCase

if __name__ == "__main__":
    logging.basicConfig(level=logging.WARN)
    unittest.main()

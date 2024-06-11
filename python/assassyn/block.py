class Block(object):

    MODULE_ROOT = 0
    CONDITIONAL = 1

    def __init__(self, kind: int):
        self.kind = kind
        self.body = []


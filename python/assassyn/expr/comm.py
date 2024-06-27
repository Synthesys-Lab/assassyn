'''The module for communitative operations'''

import operator

def reduce(op, *args):
    '''Reduce the arguments using the operator'''
    res = args[0]
    for arg in args[1:]:
        res = op(res, arg)
    return res

def add(*args):
    '''Add all the arguments'''
    return reduce(operator.add, *args)

def mul(*args):
    '''Multiply all the arguments'''
    return reduce(operator.mul, *args)

def and_(*args):
    '''Bitwise and on all the arguments'''
    return reduce(operator.and_, *args)

def or_(*args):
    '''Bitwise or on all the arguments'''
    return reduce(operator.or_, *args)

def xor(*args):
    '''Bitwise xor on all the arguments'''
    return reduce(operator.xor, *args)

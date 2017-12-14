# yo dawg.., i heard, you like code generation,,,

from collections import namedtuple

SWIG_HEADER = r'''
%module battlecode
%include "attribute.i"
//...

'''

class Type(object):
    def __init__(self, rust, swig):
        self.rust = rust
        self.swig = swig

    def to_swig(self):
        return self.swig

    def to_swig_extra(self):
        return ''

    def to_rust(self):
        return self.to_rust

    def mut_ptr(self):
        return Type('*mut '+self.rust, self.swig + '*')

u8 = Type('u8', 'uint8_t')
i8 = Type('i8', 'int8_t')
u16 = Type('u16', 'uint16_t')
i16 = Type('i16', 'int16_t')
u32 = Type('u32', 'uint32_t')
i32 = Type('i32', 'int32_t')
u64 = Type('u64', 'uint64_t')
i64 = Type('i64', 'int64_t')

class Var(object):
    def __init__(self, type, name):
        self.type = type
        self.name = name
    
    def to_swig(self):
        return '{0.type.swig} {0.name}'.format(self)

    def to_swig_extra(self):
        return ''

    def to_rust(self):
        return '{0.name}: {0.type.rust}'.format(self)

class Signature(object):
    def __init__(self, type, name, args):
        self.type = type
        self.name = name
        self.args = args
    
    def to_swig(self):
        result = '{0.type.swig} {0.name}('.format(self)
        result += ', '.join(a.to_swig() for a in self.args)
        result += ');\n'
        return result

    def to_swig_extra(self):
        return ''

    def to_rust(self, body):
        result = '#[no_mangle]\npub extern "C" fn {0.name}('.format(self)
        result += ', '.join(a.to_rust() for a in self.args)
        result += ') -> {0.type.rust} {{\n{1}\n}}\n'.format(self, body)
        return result

s = Signature(u8, 'bananas', [Var(u8, 'taekwondo'), Var(u32, 'explosion')])

print(s.to_swig())
print(s.to_rust(''))

class Struct(object):
    def __init__(self, name):
        self.name = name
        self.members = []
        self.methods = []
        self.method_names = []
        self.type = Type(name, name)
    
    def member(self, type, name):
        self.members.append(Var(type,name))
        return self

    def self_pointer(self):
        return Var(self.type.mut_ptr(), 'self')

    def method(self, type, name, args):
        self.methods.append(Signature(type, self.name + '_' + name, [self.self_pointer()] + args))
        self.method_names.append(name)
        return self

    def to_swig(self):
        '''Generate a SWIG interface for this interface.'''
        definition = 'typedef struct {0.name} {{'.format(self)
        definition += ''.join('\n\t' + member.to_swig() + ';' for member in self.members)
        definition += '\n}} {0.name};'.format(self)

        methods = ''.join(method.to_swig() for method in self.methods)

        return '{}\n{}'.format(definition, methods)

    def to_swig_extra(self):
        extra = '%extend {0.name} {{\n'.format(self)
        extra += ''.join(
            '\t' + Signature(method.type, method_name, method.args[1:]).to_swig()
            for method, method_name in zip(self.methods, self.method_names)
        )
        extra += '}};'
        return extra

    def to_rust(self):
        '''Generate a rust implementation for this struct.'''
        # assume that struct is already defined
        return ''.join(method.to_rust(
            '\t(&mut (*self)).{}('.format(method_name) +
            ', '.join('{0.name}'.format(arg) for arg in method.args[1:]) +
            ')'
        ) for method, method_name in zip(self.methods, self.method_names))

s = Struct('Banana')\
    .member(u8, 'hello')\
    .member(u32, 'goodbye')\
    .method(i64, 'peel', [Var(i16, 'amount')])\
    .method(i64, 'consume', [Var(u8, 'greedy'), Var(i16, 'solipsistic')])\
    .method(i64, 'peel_harder', [Var(i16, 'amount'), Var(u64, 'veracity')])

print(s.to_swig())
print(s.to_swig_extra())
print()
print(s.to_rust())

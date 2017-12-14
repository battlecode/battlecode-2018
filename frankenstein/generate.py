# yo dawg.., i heard, you like code generation,,,

from collections import namedtuple

SWIG_HEADER = r'''
%module battlecode
%include "getter.i"
//...

'''

class Type(object):
    def __init__(self, rust, swig):
        self.rust = rust
        self.swig = swig

    def to_swig(self):
        return self.swig

    def to_c(self):
        return self.to_swig()

    def to_rust(self):
        return self.rust

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

    def to_c(self):
        return self.to_swig()

    def to_rust(self):
        return '{0.name}: {0.type.rust}'.format(self)

class Function(object):
    def __init__(self, type, name, args, body=''):
        self.type = type
        self.name = name
        self.args = args # type: List
        self.body = body
    
    def to_swig(self):
        result = '{0.type.swig} {0.name}('.format(self)
        result += ', '.join(a.to_swig() for a in self.args)
        result += ');\n'
        return result

    def to_c(self):
        return self.to_swig()

    def to_rust(self):
        result = '#[no_mangle]\npub extern "C" fn {0.name}('.format(self)
        result += ', '.join(a.to_rust() for a in self.args)
        result += ') -> {0.type.rust} {{\n{1}\n}}\n'.format(self, self.body)
        return result

class Struct(object):
    def __init__(self, name, opaque=False):
        self.name = name
        self.members = []
        self.methods = []
        self.method_names = []
        self.getters = []
        self.type = Type(name, name)
        self.opaque = opaque
    
    def member(self, type, name):
        self.members.append(Var(type,name))
        if self.opaque:
            mut_self = self.type.mut_ptr()
            # getter and setter functions to access non-repr(C) structs
            getter = Function(type, self.name + "_get_" + name, [Var(mut_self, 'self')],
                '\t(*self).{0}.clone()'.format(name)
            )
            setter = Function(type, self.name + "_set_" + name,
                [Var(mut_self,'self'), Var(type,name)],
                '\t(*self).{0} = {0};'.format(name) 
            )
            self.getters.append(getter)
            self.getters.append(setter)

        return self

    def self_pointer(self):
        return Var(self.type.mut_ptr(), 'self')

    def method(self, type, name, args):
        self.methods.append(Function(type, self.name + '_' + name, [self.self_pointer()] + args,
            '\t(&mut (*self)).{}('.format(name) +
            ', '.join('{0.name}'.format(arg) for arg in args[1:]) +
            ')'
        ))
        self.method_names.append(name)
        return self

    def _c_struct(self):
        '''Create a 'struct' definition.'''
        if self.opaque:
            definition = 'typedef struct {0.name} {0.name};\n'.format(self)
        else:
            definition = 'typedef struct {0.name} {{'.format(self)
            definition += ''.join('\n\t{};'.format(member.to_swig()) for member in self.members)
            definition += '\n}} {0.name};\n'.format(self)
        return definition

    def to_c(self):
        definition = self._c_struct()
        methods = ''.join(method.to_swig() for method in self.methods)
        getters = ''.join(getter.to_swig() for getter in self.getters)
        return definition + methods + getters

    def to_swig(self):
        '''Generate a SWIG interface for this struct.'''
        definition = self._c_struct()
        # We use SWIG's %extend command to attach "methods" to this struct:
        # %extend Bananas {
        #     int peel(int);
        # }
        # results in a `peel` method on the Bananas object, which
        # calls into a method:
        # int Bananas_peel(Bananas *self, int)
        # which we generate :)
        extra = '%extend {0.name} {{\n'.format(self)
        for method, method_name in zip(self.methods, self.method_names):
            extra += '\t' + Function(method.type, method_name, method.args[1:]).to_swig()
        if self.opaque:
            for member in self.members:
                # add getters
                extra += '\n\t' + member.to_swig() + ';'

        extra += '\n};'

        return '{}\n{}'.format(definition, extra)

    def to_rust(self):
        '''Generate a rust implementation for this struct.'''
        # assume that struct is already defined
        result = ''.join(method.to_rust() for method in self.methods)
        result += ''.join(getter.to_rust() for getter in self.getters)

        return result

SWIG_HEADER = '''%module {module}
/// GENERATED SWIG
#include <stdint.h>
%{{
#include "{module}.h"
}}
'''

RUST_HEADER = '''/// GENERATED RUST
/// This function ensures we aren't sending any borrowed types outside of Rust,
/// where the borrow checker's guarantees can't be upheld.
fn check<T: 'static + Send>(val: T) -> T { val }

'''

class Program(object):
    def __init__(self, name):
        self.name = name
        self.elements = []
    
    def add(self, elem):
        self.elements.append(elem)
        return self

    def to_rust(self):
        return RUST_HEADER + ''.join(elem.to_rust() for elem in self.elements)

    def to_c(self):
        return '//! GENERATED C\n#include <stdint.h>\n' +\
            ''.join(elem.to_c() for elem in self.elements)

    def to_swig(self):
        result = SWIG_HEADER.format(module=self.name)
        result += ''.join(elem.to_swig() for elem in self.elements)
        return result

    def write_files(self):
        with open(self.name + '.rs', 'w') as f:
            f.write(self.to_rust())
        with open(self.name + '.h', 'w') as f:
            f.write(self.to_c())
        with open(self.name + '.i', 'w') as f:
            f.write(self.to_swig())

p = Program('speleothem')\
    .add(Function(u8, 'ziggurat', [Var(u8, 'taekwondo'), Var(u32, 'explosion')]))\
    .add(Struct('Banana', opaque=True)\
        .member(u8, 'hello')\
        .member(u32, 'goodbye')\
        .method(i64, 'peel', [Var(i16, 'amount')])\
        .method(i64, 'consume', [Var(u8, 'greedy'), Var(i16, 'solipsistic')])\
        .method(i64, 'peel_harder', [Var(i16, 'amount'), Var(u64, 'veracity')])
    )

print(p.to_rust())
print('---')
print(p.to_c())
print('---')
print(p.to_swig())

p.write_files()
use general::{Source, Span};

#[test]
fn valid() {
    let input = "
// All the Char related Stuff
char char_1;
signed char char_2;
unsigned char char_3;

// All the Short related Stuff
short short_1;
short int short_2;
signed short int short_3;
unsigned short int short_4;

// All the Int related Stuff
int int_1;
signed int int_2;
unsigned int int_3;

// All the single Long related Stuff
long long_1;
signed long long_2;
unsigned long long_3;
long int long_4;
signed long int long_5;
unsigned long int long_6;

// All the double Long related Stuff
long long dlong_1;
signed long long dlong_2;
unsigned long long dlong_3;
long long int dlong_4;
signed long long int dlong_5;
unsigned long long int dlong_6;

// All the Float related Stuff
float float_1;

// All the Double related Stuff
double double_1;
long double double_2;
    ";
    let input_source = Source::new("test", input);
    let input_span: Span = input_source.into();
    let input_tokens = tokenizer::tokenize(input_span);
    let input_ast = syntax::parse(input_tokens).unwrap();

    let result = semantic::parse(input_ast);

    assert!(result.is_ok());
}

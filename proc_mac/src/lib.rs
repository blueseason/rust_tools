//use proc_macro::Ident;
use proc_macro::TokenStream;
use quote::quote;
use syn::braced;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse_macro_input;
use syn::Ident;
use syn::LitStr;
use syn::Result;
use syn::Token;
use syn::Visibility;

#[proc_macro]
pub fn make_answer(_item: TokenStream) -> TokenStream {
    "fn answer() -> u32 { 42 }".parse().unwrap()
}

#[proc_macro_derive(AnswerFn)]
pub fn derive_answer_fn(_item: TokenStream) -> TokenStream {
    "fn answer_derive() -> u32 { 42 }".parse().unwrap()
}

#[proc_macro_derive(HelperAttr, attributes(helper))]
pub fn derive_helper_attr(_item: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_attribute]
pub fn show_streams(attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("attr: \"{}\"", attr.to_string());
    println!("item: \"{}\"", item.to_string());
    item
}

#[proc_macro_derive(HelloMacro)]
pub fn hello_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_hello_macro(&ast)
}

fn impl_hello_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl HelloMacro for #name {
            fn hello_macro() {
                println!("Hello, Macro! My name is {}!", stringify!(#name));
            }
        }
    };
    gen.into()
}
/*
 生成类似如下代码
pub struct MyStaticMetric { foo : MyType, bar : MyType, } impl MyStaticMetric
{
    pub fn new() -> Self
    {
        Self
        {
        foo : MyType :: with_label_values("foo"),
        bar : MyType :: with_label_values("bar"),
        }
    }
}
 */
#[proc_macro]
pub fn make_metrics(input: TokenStream) -> TokenStream {
    let data = parse_macro_input!(input as MetricDefinition);
    //    println!("{:?}", data.name);
    //    println!("{:?}", data.values);
    let vis = &data.vis;
    let name = &data.name;
    let values = &data.values;
    let values_str = data
        .values
        .iter()
        .map(|ident| LitStr::new(&ident.to_string(), ident.span()));
    let expanded = quote! {
        #vis struct #name {
            #(#values: MyType,)*
        }
        impl #name {
            pub fn new() -> Self {
                Self {
                    #(#values: MyType::with_label_values(
                        #values_str),)*
                }
            }
        }
    };
    println!("{}", expanded);
    TokenStream::from(expanded)
}

//#[derive(Debug)]
struct MetricDefinition {
    vis: Visibility,
    name: Ident,
    values: Vec<Ident>,
}

//span 表示源码中的位置范围（即 语法树节点在原始源代码中的起始和结束位置）
// span: #0 bytes(17..20)：
//       17: foo 在文件中的起始字节偏移。
//       20: foo 在文件中的结束字节偏移（不包含）。
//       #0 表示这是主源文件中的代码。
impl Parse for MetricDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let vis = input.parse::<Visibility>()?;
        input.parse::<Token![struct]>()?;
        let name = input.parse::<Ident>()?;
        let content;
        braced!(content in input);
        let values = content
            .parse_terminated(Ident::parse, Token![,])?
            .into_iter()
            .collect();
        Ok(Self { vis, name, values })
    }
}

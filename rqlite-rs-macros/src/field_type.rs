use syn::Type;

pub(crate) enum FieldType {
    Option,
    Blob,
    Normal,
}

impl FieldType {
    pub(crate) fn from_type_path(type_path: &syn::TypePath) -> Self {
        if type_path.path.segments.last().unwrap().ident == "Option" {
            return Self::Option;
        }

        #[cfg(feature = "blob")]
        if is_vec_type(type_path) && has_u8_generic_arg(type_path) {
            return Self::Blob;
        }

        Self::Normal
    }
}

fn is_vec_type(type_path: &syn::TypePath) -> bool {
    type_path
        .path
        .segments
        .last()
        .map_or(false, |seg| seg.ident == "Vec")
}

fn has_u8_generic_arg(type_path: &syn::TypePath) -> bool {
    let Some(last_segment) = type_path.path.segments.last() else {
        return false;
    };

    match &last_segment.arguments {
        syn::PathArguments::AngleBracketed(args) => args.args.first().map_or(false, |arg| {
            if let syn::GenericArgument::Type(Type::Path(inner_path)) = arg {
                inner_path
                    .path
                    .segments
                    .last()
                    .map_or(false, |seg| seg.ident == "u8")
            } else {
                false
            }
        }),
        _ => false,
    }
}

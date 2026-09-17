fn main() {
    #[cfg(windows)]
    {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("assets/livro-caixa-rv.ico");
        resource.compile().expect("falha ao incorporar o ícone");
    }
}

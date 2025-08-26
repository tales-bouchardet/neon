# NEON — Tecnologias, Lógicas e Bibliotecas

**NEON** é um projeto em Rust para diagnóstico e monitoramento de sistemas Windows, com foco em performance, simplicidade e uso eficiente dos recursos nativos do sistema.

---

## Lógicas Win32 Utilizadas

- **Carregamento Dinâmico de DLLs:**  
  O projeto utiliza `LoadLibraryW` para carregar dinamicamente a `dsreg.dll`, permitindo consultar o status de associação do dispositivo ao domínio (Active Directory) ou ao Intune. As funções da DLL são acessadas via `GetProcAddress`, tornando o código flexível e compatível com diferentes versões do Windows.

- **Cacheamento com Mutex e OnceLock:**  
  Para evitar consultas repetidas e garantir segurança em ambientes concorrentes, NEON utiliza caches protegidos por `Mutex` e inicializados com `OnceLock`. Cada função que realiza operações pesadas (como acesso ao registro, rede ou APIs do Windows) armazena o resultado em cache junto com o timestamp da última atualização. Assim, as informações são atualizadas apenas após um intervalo definido (TTL), tornando as requisições “lazy” — só são feitas quando realmente necessário.

- **Requisição Lazy:**  
  Todas as funções que consultam o sistema, registro ou rede só executam a operação pesada se o cache estiver expirado, retornando instantaneamente o valor armazenado caso contrário. Isso garante alta performance e baixo consumo de recursos.

---

## Bibliotecas e APIs Utilizadas

- **windows:**  
  Permite acesso direto às APIs nativas do Windows, como consultas de hardware, sistema operacional, firewall, proxy, memória, CPU e rede.

- **winreg:**  
  Facilita a leitura eficiente do registro do Windows, essencial para detectar softwares instalados e obter informações detalhadas do sistema.

- **Firewall:**  
  Utiliza as APIs do Windows (`INetFwPolicy2`, `NetFwPolicy2`) para consultar o status do firewall de forma nativa, sem depender de comandos externos.

- **Proxy:**  
  Usa a API `WinHttpGetIEProxyConfigForCurrentUser` para verificar se há proxy configurado no sistema, acessando diretamente as configurações do Internet Explorer/Windows.

---

## Verificação de Rede Interna

- **Ping.exe:**  
  Para checar o acesso à rede interna da empresa, NEON executa o comando `ping.exe` de forma silenciosa (sem abrir janela) usando a flag `CREATE_NO_WINDOW` via `CommandExt`. O resultado do comando indica se o host corporativo está acessível, permitindo identificar rapidamente se há conexão com a intranet.

---

## Foco do Projeto

- **Performance:**  
  Todas as operações são otimizadas para serem rápidas e discretas, evitando travamentos e uso excessivo de recursos.
- **Simplicidade:**  
  O código é enxuto, fácil de manter e pensado para funcionar em segundo plano, sem interferir na experiência do usuário.
- **Diagnóstico Completo:**  
  NEON entrega informações precisas sobre o ambiente Windows, incluindo status de domínio, firewall, proxy, memória, CPU, softwares instalados e conectividade de rede interna.

---

> O NEON foi criado para quem precisa de diagnósticos ágeis, confiáveis e com o menor impacto

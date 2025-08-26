# N.E.O.N.

**NEON** é um projeto em Rust para facilitar o diagnóstico e monitoramento de alguns dados do Windows, com foco em performance, simplicidade e uso eficiente dos recursos nativos do sistema.

---

## Lógicas Win32 Utilizadas

- **Cacheamento com Mutex e OnceLock:**  
   Para evitar consultas repetidas e garantir segurança em ambientes concorrentes, NEON utiliza caches protegidos por `Mutex` e inicializados com `OnceLock`. Cada função que realiza operações pesadas (como acesso ao registro, rede ou APIs do Windows) armazena o resultado em cache junto com o timestamp da última atualização. Assim, as informações são atualizadas apenas após um intervalo definido (TTL), tornando as requisições “lazy” — só são feitas quando realmente necessário.

## Bibliotecas e APIs Utilizadas

- **windows:**  
  Permite acesso direto às APIs nativas do Windows, como consultas de hardware, sistema operacional, firewall, proxy, memória, CPU e rede.

- **winreg:**  
  Facilita a leitura eficiente do registro do Windows, essencial para detectar softwares instalados e obter informações detalhadas do sistema.

- **Firewall:**  
  Utiliza as APIs do Windows (`INetFwPolicy2`, `NetFwPolicy2`) para consultar o status do firewall de forma nativa.

- **Proxy:**  
  Usa a API `WinHttpGetIEProxyConfigForCurrentUser` para verificar se há proxy configurado no sistema, acessando diretamente as configurações do Internet Explorer/Windows.

- **Carregamento Dinâmico de DLLs:**  
 O projeto utiliza `LoadLibraryW` para carregar dinamicamente a `dsreg.dll`, permitindo consultar o gerenciamento do dispositivo ao domínio (Active Directory), ou ao Intune.
---

## Verificação de Rede Interna

- **Ping.exe:**  
  Para checar o acesso à rede interna da empresa, NEON executa o comando `ping.exe` de forma silenciosa.

---

## Foco do Projeto

- **Performance:**  
  Todas as operações são otimizadas para serem rápidas e discretas, evitando travamentos e uso excessivo de recursos.
  
- **Simplicidade:**  
  O código é enxuto, fácil de ler e manter.
  
- **Diagnóstico Completo:**  
  NEON entrega informações precisas sobre o ambiente Windows, incluindo status de domínio, firewall, proxy, memória, CPU, softwares instalados e conectividade de rede interna.

---

> O programa foi criado para atender a necessidades operacionais específicas para documentar de forma fácil alguns dados de diagnostico nas máquinas de uma empresa específica.

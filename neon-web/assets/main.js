function updateData() {
    if (external.invoke("refresh")) {
        window.alert("Erro ao atualizar os dados.");
    }

    document.querySelectorAll('.date-time')[0].textContent = data.now;
    document.querySelectorAll('.date-time')[1].textContent = data.date;
    document.querySelector('#manufacturer .data').textContent = data.manufacturer;
    document.querySelector('#os .data').textContent = data.os;
    document.querySelector('#user .data').textContent = data.user;
    document.querySelector('#hostname .data').textContent = data.hostname;
    document.querySelector('#cpu .data').textContent = data.cpu;
    document.querySelector('#cpu_usage .data').textContent = data.cpu_usage;

    let mem = JSON.parse(data.memory);
    document.querySelector('#ram .data').textContent =  mem[0];
    document.querySelector('#ram_free .data').textContent =  mem[1];

    document.querySelector('#uptime .data').textContent = data.uptime;

    document.querySelector('#join').innerHTML = data.join;
    document.querySelector('#proxy').innerHTML = data.proxy;
    document.querySelector('#intranet').innerHTML = data.intranet;

    document.querySelector('#firewall .data').textContent = data.firewall;
    if (data.firewall !== "Ativado") {
        document.querySelector('#firewall .label').style.color = "#00b86b";
    }

    document.querySelector('#forti .data').textContent = data.forticlient;
    if (data.forticlient !== "Não Instalado") {
        document.querySelector('#forti .label').style.color = "#00b86b";
    }

    document.querySelector('#crowdstrike .data').textContent = data.crowd;
    if (data.crowd !== "Não Instalado") {
        document.querySelector('#crowdstrike .label').style.color = "#00b86b";
    }

    document.querySelector('#kace .data').textContent = data.kace;
    if (data.kace !== "Não Instalado") {
        document.querySelector('#kace .label').style.color = "#00b86b";
    }
    
    document.querySelector('#netskope .data').textContent = data.netskope;
    if (data.netskope !== "Não Instalado") {
        document.querySelector('#netskope .label').style.color = "#00b86b";
    }
};

external.invoke("refresh");
updateData();
setInterval(updateData, 900);
export function main() {
    let plots = [];

    m.request({
        method: 'GET',
        url: '/probes',
    }).then(result => {
        result.probes.forEach(probe =>
            [100, 1000, 10000, 50000].forEach(speed =>
                plots.push({
                    params: {
                        probe,
                        protocol_speed: speed,
                    }
                }))
        )

        m.mount(document.body, {
            view(vnode) {
                return m('.container', [
                    m('row', m('.col', m('h2.text-center.m-4', 'probe-rs performance & regression tracking'))),
                    m('.row', plots.map((p, i) => m('.col-6.plot.mt-5', m('canvas', {
                        oninit(vnode) {
                            vnode.state.plot = undefined;
                            vnode.state.resize = function () { };
                        },
                        oncreate(vnode) {
                            vnode.state.dom = vnode.dom;
                            createPlot(plots[i], vnode.dom).then(plot => {
                                vnode.state.plot = plot;

                                vnode.state.resize = function () {
                                    // if (plot) {
                                    //     plot.setSize({
                                    //         width: vnode.dom.parent.clientWidth,
                                    //         height: vnode.dom.parent.clientWidth / 1.41,
                                    //     })
                                    // }
                                    m.redraw()
                                }

                                window.addEventListener('resize', vnode.state.resize)
                            });
                        },
                        onremove(vnode) {
                            window.removeEventListener('resize', vnode.state.resize)
                        },
                        // width: vnode.state.dom ? vnode.state.dom.parent.clientWidth : 0,
                        // height: vnode.state.dom ? vnode.state.dom.parent.clientWidth / 1.41 : 0
                    }))))
                ])
            }
        })
    });
}

/// https://stackoverflow.com/questions/15900485/correct-way-to-convert-size-in-bytes-to-kb-mb-gb-in-javascript
function formatBytes(bytes, decimals = 2) {
    if (bytes === 0) return '0 Bytes';

    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];

    const i = Math.floor(Math.log(bytes) / Math.log(k));

    return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
}

function formatHz(bytes, decimals = 2) {
    if (bytes === 0) return '0 Hz';

    const k = 1000;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['Hz', 'KHz', 'MHz', 'GHz', 'THz'];

    const i = Math.floor(Math.log(bytes) / Math.log(k));

    return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
}

function createPlot(plot, dom) {
    return m.request({
        method: 'GET',
        url: '/list',
        params: plot.params
    }).then(function (result) {
        let config = {
            type: 'line',
            data: {
                labels: result.logs.map(v => v.commit_hash),
                datasets: [
                    {
                        label: 'Read',
                        borderColor: 'rgb(255, 99, 132)',
                        fill: false,
                        data: result.logs.map(v => v.read_speed),
                    },
                    {
                        label: 'Write',
                        borderColor: 'rgb(54, 162, 235)',
                        fill: false,
                        data: result.logs.map(v => v.write_speed),
                    },
                    {
                        label: 'abs ΔRead',
                        borderColor: 'rgb(54, 162, 235)',
                        showLine: false,
                        fill: false,
                        pointRadius: 0,
                        pointHoverRadius: 0,
                        yAxisID: 'y-axis-2',
                        data: result.logs.map(v => v.read_speed / result.logs[0].read_speed * 100 - 100),
                    },
                    {
                        label: 'abs ΔWrite',
                        borderColor: 'rgb(54, 162, 235)',
                        showLine: false,
                        fill: false,
                        pointRadius: 0,
                        pointHoverRadius: 0,
                        yAxisID: 'y-axis-2',
                        data: result.logs.map(v => v.write_speed / result.logs[0].write_speed * 100 - 100),
                    },
                    {
                        label: 'ΔRead',
                        borderColor: 'rgb(54, 162, 235)',
                        showLine: false,
                        fill: false,
                        pointRadius: 0,
                        pointHoverRadius: 0,
                        yAxisID: 'y-axis-2',
                        data: result.logs.map((v, i) => v.read_speed / result.logs[Math.max(i - 1, 0)].read_speed * 100 - 100),
                    },
                    {
                        label: 'ΔWrite',
                        borderColor: 'rgb(54, 162, 235)',
                        showLine: false,
                        fill: false,
                        pointRadius: 0,
                        pointHoverRadius: 0,
                        yAxisID: 'y-axis-2',
                        data: result.logs.map((v, i) => v.write_speed / result.logs[Math.max(i - 1, 0)].write_speed * 100 - 100),
                    },
                ]
            },
            options: {
                title: {
                    text: plot.params.probe + ': ' + formatHz(plot.params.protocol_speed * 1e3),
                    display: true,
                },
                legend: {
                    labels: {
                        filter: (item, chart) => item.datasetIndex < 2
                    }
                },
                scales: {
                    yAxes: [{
                        ticks: {
                            callback: (value, index, values) => formatBytes(value)
                        }
                    }, {
                        type: 'linear',
                        display: false,
                        position: 'right',
                        id: 'y-axis-2',
                        gridLines: {
                            drawOnChartArea: false,
                        },
                    }]
                },
                tooltips: {
                    position: 'nearest',
                    mode: 'index',
                    callbacks: {
                        label: (tooltipItem, data) => {
                            let prefix = config.data.datasets[tooltipItem.datasetIndex].label;
                            if (tooltipItem.datasetIndex > 1) {
                                return prefix + ': ' + parseFloat(tooltipItem.value).toFixed(2) + '%'
                            } else {
                                return prefix + ': ' + formatBytes(tooltipItem.value)
                            }
                        },
                    }
                }
            }
        }

        var ctx = dom.getContext('2d');
        return new Chart(ctx, config);
    });
}
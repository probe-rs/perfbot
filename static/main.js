import uPlot from '/static/uPlot.esm.js';

export function main() {
    let plots = [];

    m.request({
        method: 'GET',
        url: '/setups',
    }).then(result => {
        result.setups.forEach(([probe, chip]) =>
            plots.push({
                params: {
                    probe,
                    chip,
                }
            })
        )

        m.mount(document.body, {
            view(vnode) {
                return m('.container', [
                    m('row', m('.col', m('h2.text-center.m-4', 'probe-rs performance & regression tracking'))),
                    m('.row', plots.map((p, i) => m('.col-6.plot.mt-5', {
                        oninit(vnode) {
                            vnode.state.plot = undefined;
                            vnode.state.resize = function () { };
                        },
                        oncreate(vnode) {
                            createPlot(plots[i], vnode.dom).then(plot => {
                                vnode.state.plot = plot;

                                vnode.state.resize = function () {
                                    if (plot) {
                                        plot.setSize({
                                            width: vnode.dom.clientWidth,
                                            height: vnode.dom.clientWidth / 1.41,
                                        })
                                    }
                                }

                                window.addEventListener('resize', vnode.state.resize)
                            });
                        },
                        onremove(vnode) {
                            window.removeEventListener('resize', vnode.state.resize)
                        }
                    }))
                    )
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

function createPlot(plot, dom) {
    return m.request({
        method: 'GET',
        url: '/list',
        params: plot.params
    }).then(function (result) {
        let data = [
            result.logs.map(
                (v, i) => i),
            result.logs.map(v => v.read_speed),
            result.logs.map(v => v.write_speed),
            result.logs.map(v => v.read_speed / result.logs[0].read_speed * 100 - 100),
            result.logs.map(v => v.write_speed / result.logs[0].write_speed * 100 - 100),
            result.logs.map((v, i) => v.read_speed / result.logs[Math.max(i - 1, 0)].read_speed * 100 - 100),
            result.logs.map((v, i) => v.write_speed / result.logs[Math.max(i - 1, 0)].write_speed * 100 - 100)
        ];
        const opts = {
            width: dom.clientWidth,
            height: dom.clientWidth / 1.41,
            title: plot.params.probe + ': ' + plot.params.chip,
            axes: [
                {
                    values: (self, vals) => vals.map(i => result.logs[i].commit_hash),
                    space: 100,
                    rotate: -90,
                    size: 100,
                }, {
                    values: (self, vals) => vals.map(b => formatBytes(b)),
                    scale: 'b',
                    size: 70,
                }, {
                    values: (self, vals) => vals.map(b => b.toFixed(2) + '%'),
                    side: 1,
                    scale: '%',
                    size: 70,
                }
            ],
            scales: {
                x: {
                    date: false,
                    distr: 2,
                }
            },
            series: [
                {
                    label: 'commit',
                    value: (self, i) => result.logs[i].commit_hash
                }, {
                    stroke: 'orange',
                    label: 'read',
                    value: (self, b) => formatBytes(b),
                    scale: 'b',
                }, {
                    stroke: 'red',
                    label: 'write',
                    value: (self, b) => formatBytes(b),
                    scale: 'b',
                }, {
                    stroke: 'orange',
                    dash: [10, 5],
                    label: 'abs Δread',
                    value: (self, b) => b.toFixed(2) + '%',
                    scale: '%',
                }, {
                    stroke: 'red',
                    dash: [10, 5],
                    label: 'abs Δwrite',
                    value: (self, b) => b.toFixed(2) + '%',
                    scale: '%',
                }, {
                    stroke: 'orange',
                    dash: [5, 10],
                    label: 'Δread',
                    value: (self, b) => b.toFixed(2) + '%',
                    scale: '%',
                }, {
                    stroke: 'red',
                    dash: [5, 10],
                    label: 'Δwrite',
                    value: (self, b) => b.toFixed(2) + '%',
                    scale: '%',
                }
            ],
        };
        return new uPlot(opts, data, dom);
    });
}
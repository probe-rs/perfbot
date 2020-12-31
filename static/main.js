import uPlot from '/static/uPlot.esm.js';

export function main() {
    let plots = [{
        params: {
            probe: 'CMSIS-DAP',
            chip: 'nRF51822',
        }
    },
    {
        params: {
            probe: 'J-Link',
            chip: 'nRF52840',
        }
    }
    ];

    m.mount(document.body, {
        view(vnode) {
            return m('.container', [
                m('.row', plots.map((p, i) => m('.col-4.plot', {
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
}

function tooltipPlugin({ logs, absoluteMode, shiftX = 10, shiftY = 10 }) {
    let tooltipLeftOffset = 0;
    let tooltipTopOffset = 0;

    const tooltip = document.createElement('div');
    tooltip.className = 'u-tooltip';

    let seriesIdx = null;
    let dataIdx = null;

    const fmtDate = uPlot.fmtDate('{M}/{D}/{YY} {h}:{mm}:{ss} {AA}');

    let over;

    let tooltipVisible = false;

    function showTooltip() {
        console.log('kek')
        if (!tooltipVisible) {
            tooltip.style.display = 'block';
            over.style.cursor = 'pointer';
            tooltipVisible = true;
        }
    }

    function hideTooltip() {
        if (tooltipVisible) {
            tooltip.style.display = 'none';
            over.style.cursor = null;
            tooltipVisible = false;
        }
    }

    function setTooltip(u) {
        showTooltip();

        let top = u.valToPos(u.data[seriesIdx][dataIdx], 'y');
        let left = u.valToPos(u.data[0][dataIdx], 'x');

        tooltip.style.top = (tooltipTopOffset + top + shiftX) + 'px';
        tooltip.style.left = (tooltipLeftOffset + left + shiftY) + 'px';

        let trailer = '';
        if (absoluteMode) {
            let pctSinceStart = (((u.data[seriesIdx][dataIdx] - u.data[seriesIdx][0]) / u.data[seriesIdx][0]) * 100).toFixed(2);
            trailer = uPlot.fmtNum(u.data[seriesIdx][dataIdx]) + ' (' +
                pctSinceStart + '% since start)';
        } else {
            trailer = uPlot.fmtNum(u.data[seriesIdx][dataIdx]) + '% since start';
        }
        tooltip.textContent = (
            fmtDate(new Date(u.data[0][dataIdx] * 1e3)) + ' - ' +
            logs[dataIdx][1].slice(0, 10) + '\n' + trailer
        );
    }

    return {
        hooks: {
            ready: [
                u => {
                    console.log('kek')
                    over = u.root.querySelector('.u-over');

                    tooltipLeftOffset = parseFloat(over.style.left);
                    tooltipTopOffset = parseFloat(over.style.top);
                    u.root.querySelector('.u-wrap').appendChild(tooltip);

                    let clientX;
                    let clientY;

                    over.addEventListener('mousedown', e => {
                        clientX = e.clientX;
                        clientY = e.clientY;
                    });

                    over.addEventListener('mouseup', e => {
                        // clicked in-place
                        if (e.clientX == clientX && e.clientY == clientY) {
                            if (seriesIdx != null && dataIdx != null) {
                                onclick(u, seriesIdx, dataIdx);
                            }
                        }
                    });
                }
            ],
            setCursor: [
                u => {
                    console.log('kek')
                    let c = u.cursor;

                    if (dataIdx != c.idx) {
                        dataIdx = c.idx;

                        if (seriesIdx != null)
                            setTooltip(u);
                    }
                }
            ],
            setSeries: [
                (u, sidx) => {
                    console.log('kek')
                    if (seriesIdx != sidx) {
                        seriesIdx = sidx;

                        if (sidx == null)
                            hideTooltip();
                        else if (dataIdx != null)
                            setTooltip(u);
                    }
                }
            ],
        }
    };
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
            result.logs.map(v => v.write_speed)
        ];
        const opts = {
            width: dom.clientWidth,
            height: dom.clientWidth / 1.41,
            title: plot.params.probe + ': ' + plot.params.chip,
            tzDate: ts => uPlot.tzDate(new Date(ts * 1e3), 'Etc/UTC'),
            axes: [
                {
                    values: (self, vals) => vals.map(i => result.logs[i].commit_hash),
                    space: 100,
                    rotate: -90,
                    size: 100,
                }, {}
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
                    label: 'read'
                }, {
                    stroke: 'red',
                    label: 'write'
                }
            ],
            plugins: [
                {
                    hooks: {
                        drawAxes: [
                            u => {
                                let { ctx } = u;
                                let { left, top, width, height } = u.bbox;

                                const interpolatedColorWithAlpha = '#fcb0f15f';

                                ctx.strokeStyle = interpolatedColorWithAlpha;
                                ctx.beginPath();

                                let [i0, i1] = u.series[0].idxs;

                                for (let j = i0; j <= i1; j++) {
                                    let v = u.data[0][j];
                                }

                                ctx.closePath();
                                ctx.stroke();
                            },
                        ]
                    },
                },
                tooltipPlugin({
                    logs: result.logs,
                    absoluteMode: false,
                }),
            ],
        };
        return new uPlot(opts, data, dom);
    });
}
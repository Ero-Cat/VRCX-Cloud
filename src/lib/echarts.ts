import { BarChart, HeatmapChart, LineChart } from 'echarts/charts';
import {
    GridComponent,
    TooltipComponent,
    VisualMapComponent
} from 'echarts/components';
import * as echarts from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';

echarts.use([
    BarChart,
    LineChart,
    HeatmapChart,
    GridComponent,
    TooltipComponent,
    VisualMapComponent,
    CanvasRenderer
]);

export { echarts };

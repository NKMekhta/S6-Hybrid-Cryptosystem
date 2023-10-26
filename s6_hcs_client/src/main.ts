import { createApp } from "vue";
import App from "./App.vue";

import 'vuetify/styles'
import { createVuetify } from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'
import {mdi, aliases} from "vuetify/iconsets/mdi";
import '@mdi/font/css/materialdesignicons.css'


const vuetify = createVuetify({
    components,
    directives,
    theme: {
        defaultTheme: 'dark',
    },
    icons: {
        defaultSet: 'mdi',
        aliases,
        sets: {
            mdi
        }
    }
})

const app = createApp(App).use(vuetify);
app.config.globalProperties.$eventBus = app;
app.mount('#app');

<template>
  <v-app>
    <v-toolbar title="S6 Hybrid Cryptosystem" color="dark">
      <v-tabs stacked v-model="tab">

        <v-tab value="file-mgr">
          <v-icon>mdi-file-key</v-icon>
          File Manager
        </v-tab>

        <v-tab value="operations">
          <v-icon>mdi-play-speed</v-icon>
          Operations
        </v-tab>

        <v-tab value="connection">
          <v-icon>mdi-connection</v-icon>
          Connection
        </v-tab>

      </v-tabs>
    </v-toolbar>

    <v-window v-model="tab">


      <v-window-item value="file-mgr">
        <v-btn @click="handleUpload">
          <v-icon>mdi-upload</v-icon>
          Upload File
        </v-btn>
        <v-btn @click="handleRefresh">
          <v-progress-circular
              v-if="isRefreshing"
              indeterminate
              size="24"
          ></v-progress-circular>
          <span v-else>
            <v-icon>mdi-refresh-circle</v-icon>
            Refresh
          </span>
        </v-btn>
        <v-data-table-virtual
            :headers="file_headers"
            :items="file_items"
            item-value="id"
            pagination="false"
        >
          <template v-slot:item.actions="{ item }">
            <v-btn @click="() => handleDelete(item)">
              <v-icon>mdi-delete</v-icon>
            </v-btn>
            <v-btn @click="() => handleDownload(item)">
              <v-icon>mdi-download</v-icon>
            </v-btn>
          </template>
        </v-data-table-virtual>
      </v-window-item>


      <v-window-item value="operations">
        <v-data-table-virtual
            :headers="op_headers"
            :items="op_items"
            item-value="id"
            pagination="false"
        >
          <template v-slot:item.status="{ item }">
            <v-progress-linear height="10" :model-value="item.progress">
            </v-progress-linear>
            {{ item.status }}, {{ item.progress }}%
          </template>
        </v-data-table-virtual>
      </v-window-item>


      <v-window-item value="connection">
        <v-form>
          <v-container>
            <v-row>
              <v-col cols="12" md="4">
                <v-text-field
                    v-model="hostname"
                    label="Server address"
                    required
                    hide-details
                ></v-text-field>
              </v-col>

              <v-col cols="12" md="4">
                <v-text-field
                    v-model="port"
                    type="number"
                    :counter="5"
                    label="Port number"
                    hide-details
                    required
                ></v-text-field>
              </v-col>

            </v-row>
          </v-container>
        </v-form>
      </v-window-item>


    </v-window>

  </v-app>
</template>


<script setup lang="ts">
import {VDataTableVirtual} from "vuetify/labs/VDataTable";
import {ref, computed, reactive} from "vue";
import {createVuetify} from "vuetify";
import { open, save } from '@tauri-apps/api/dialog';
import {invoke} from "@tauri-apps/api/tauri";
import {filesize} from "filesize";
import { listen } from '@tauri-apps/api/event';



createVuetify({
  components: {
    VDataTableVirtual,
  },
});


const hostname = ref("localhost");
const port = ref(2794);
const tab = ref();
const isRefreshing = ref(false);
let address = computed(() => {
  return "ws://" + hostname.value + ':' + port.value.toString();
})


let file_headers = ref([
  { title: 'File Name', key: 'name' },
  { title: 'Size', key: 'size' },
  { title: '', key: 'actions', sortable: false, align: "end" },
]);
let file_items = ref([]);


let op_headers = ref([
  { title: 'File Name', key: 'name' },
  { title: 'Status', key: 'status', align: "end" },
]);
let op_items = ref([]);


const handleRefresh = async () => {
  isRefreshing.value = true;
  invoke('get_files', { url: address.value }).then((files) => {
    let new_items = [];
    for (let f in files) {
      new_items.push({
        id: files[f][0],
        size: filesize(files[f][1]),
        name: files[f][2],
      });
    }
    file_items.value = new_items;
    isRefreshing.value = false;
  }).catch((err) => {
    file_items.value = [];
    // TODO error notify
    isRefreshing.value = false;
  })
}


const handleUpload = async () => {
  const file = await open({
    multiple: false,
  });

  const ev_name = Math.random().toString(36).slice(2, 7);
  let item = reactive({
    id: ev_name,
    name: "Upload " + file,
    status: "Starting",
    progress: 0,
  });
  op_items.value.push(item);
  const unlisten = await listen(ev_name, (event) => {
    if (typeof event.payload === "string") {
      item.status = event.payload;
    } else if (event.payload instanceof Object) {
      item.status = Object.keys(event.payload)[0];
      item.progress = <number>event.payload[item.status];
    }
  });

  invoke('upload', {
    url: address.value,
    file: file.toString(),
    event: ev_name,
  }).then(() => {
    unlisten();
    item.status = "Done";
    handleRefresh();
  }).catch((err) => {
    unlisten();
    item.status = err
  })
}


const handleDownload = async (entry) => {
  let file = await save({
    defaultPath: "./" + entry.name
  });

  const ev_name = Math.random().toString(36).slice(2, 7);
  let item = reactive({
    id: ev_name,
    name: "Download " + file,
    status: "Starting",
    progress: 0,
  });
  op_items.value.push(item);
  const unlisten = await listen(ev_name, (event) => {
    if (typeof event.payload === "string") {
      item.status = event.payload;
    } else if (event.payload instanceof Object) {
      item.status = Object.keys(event.payload)[0];
      item.progress = <number>event.payload[item.status];
    }
  });
  invoke('download', {
    url: address.value,
    id: entry.id,
    file: file,
    event: ev_name,
  }).then(() => {
    unlisten();
    item.status = "Done";
    handleRefresh();
  }).catch((err) => {
    unlisten();
    item.status = err
  })
}


const handleDelete = async (entry) => {
  invoke('delete', {
    url: address.value,
    id: entry.id
  }).then(() => {
    handleRefresh()
  }).catch((err) => {
    // TODO error notify
  })
}

handleRefresh();
</script>


<style lang="sass">

</style>
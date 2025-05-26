import { Component, OnInit } from '@angular/core';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

interface Device {
  id: number;
  name: string;
  mac: string;
}
@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.css']
})
export class AppComponent implements OnInit {
   serverUrl: string | null = null;
  devices: Device[] = [];
  error: boolean | null = null;
  loading = true;
  ngOnInit() {
      this.getMacAddress().then(address => {
        console.log(address);
        this.error = false;
        this.loading = false;
      })
      .catch(err => {
        console.error(err);
        this.error = true;
         this.loading = true;
      });
  }
    async getMacAddress() {
    this.loading = true;
    const devices: Device[] = await invoke("get_mac_address");
    this.devices=devices;
    this.loading = false;;
    return devices;
  }
}

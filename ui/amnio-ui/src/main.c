#include <stdio.h>
#include "amnio_ui.h"

int main(void)
{
    printf("Initializing LVGL...\n");
    lvgl_setup(); // Call setup from amnio-ui.c

    while (1)
    {
        lvgl_update(); // Call update regularly
    }

    return 0;
}

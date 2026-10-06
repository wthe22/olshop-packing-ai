import packing.orders


def test_package_imports():
    assert packing.orders.PACKING_STATUS == ("Perlu dikirim", "Menunggu pengambilan")
